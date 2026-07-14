# TUI Renderer Implementation Plan

This document covers the implementation of a terminal UI renderer for the unode AST.
The renderer is split across three distinct layers: the Notcurses FFI bindings (TypeScript),
the unode TUI renderer (TypeScript), and the Deno host application that wires plugins,
sandboxing, and the renderer together.

---

## Architecture overview

```
Deno host process (main thread)
├── PluginHost
│   ├── Worker: plugin A  (isolated, permission-gated)
│   ├── Worker: plugin B  (isolated, permission-gated)
│   └── ...
├── TuiRenderer (TypeScript)
│   ├── LayoutEngine  (TypeScript — terminal-aware flexbox)
│   ├── NodePainter   (TypeScript — translates CanonicalNode → ncplane calls)
│   ├── FocusManager  (TypeScript — keyboard navigation state)
│   └── NotcursesFFI  (TypeScript — thin FFI bindings to libnotcurses)
│       └── libnotcurses.so  (native, loaded via Deno.dlopen)
└── StateStore (TypeScript — per-screen reactive store)
```

The renderer is **single-threaded**. All Notcurses calls happen on the main Deno
thread. Plugins run in Workers (separate V8 contexts) and communicate with the renderer
via `postMessage`. No Notcurses function is ever called from a Worker — FFI and terminal
I/O are strictly main-thread concerns.

This constraint is not arbitrary. Notcurses is explicitly not thread-safe for concurrent
rendering: calling `notcurses_render` while another thread modifies planes produces
undefined behavior. Single-threaded rendering eliminates this class of bug entirely.

---

## Part 1 — Notcurses FFI bindings

### 1.1  Loading the library

```typescript
// src/ffi/notcurses.ts

const LIB_PATH = Deno.env.get("NOTCURSES_LIB") ?? (() => {
  switch (Deno.build.os) {
    case "linux":  return "/usr/lib/libnotcurses-core.so.3";
    case "darwin": return "/usr/local/lib/libnotcurses-core.3.dylib";
    default: throw new Error(`Unsupported platform: ${Deno.build.os}`);
  }
})();

const lib = Deno.dlopen(LIB_PATH, {
  // ── Lifecycle ────────────────────────────────────────────────────────────
  notcurses_init:  { parameters: ["pointer", "pointer"], result: "pointer" },
  notcurses_stop:  { parameters: ["pointer"], result: "i32" },
  notcurses_render: { parameters: ["pointer"], result: "i32" },

  // ── Terminal dimensions ───────────────────────────────────────────────────
  notcurses_stddim_yx: {
    parameters: ["pointer", "pointer", "pointer"],
    result: "pointer"  // returns standard plane
  },

  // ── Plane management ─────────────────────────────────────────────────────
  ncplane_create:   { parameters: ["pointer", "pointer"], result: "pointer" },
  ncplane_destroy:  { parameters: ["pointer"], result: "void" },
  ncplane_move_yx:  { parameters: ["pointer", "i32", "i32"], result: "i32" },
  ncplane_resize:   {
    parameters: ["pointer", "i32", "i32", "u32", "u32", "i32", "i32", "u32", "u32"],
    result: "i32"
  },
  ncplane_erase:    { parameters: ["pointer"], result: "void" },
  ncplane_set_scrolling: { parameters: ["pointer", "u32"], result: "u32" },

  // ── Text output ───────────────────────────────────────────────────────────
  ncplane_putstr_yx:     { parameters: ["pointer", "i32", "i32", "buffer"], result: "i32" },
  ncplane_putstr_aligned: {
    parameters: ["pointer", "i32", "i32", "buffer"],
    result: "i32"
  },
  ncplane_set_fg_rgb8: { parameters: ["pointer", "u32", "u32", "u32"], result: "i32" },
  ncplane_set_bg_rgb8: { parameters: ["pointer", "u32", "u32", "u32"], result: "i32" },
  ncplane_set_styles:  { parameters: ["pointer", "u32"], result: "void" },

  // ── Pixel geometry ────────────────────────────────────────────────────────
  ncplane_pixel_geom: {
    // out params: pxy, pxx, celldimy, celldimx, maxbmapy, maxbmapx
    parameters: ["pointer", "pointer", "pointer", "pointer", "pointer", "pointer", "pointer"],
    result: "void"
  },

  // ── Images ────────────────────────────────────────────────────────────────
  ncvisual_from_file:   { parameters: ["buffer"], result: "pointer" },
  ncvisual_from_memory: { parameters: ["pointer", "u64", "u32", "u32", "u32"], result: "pointer" },
  ncvisual_blit:        { parameters: ["pointer", "pointer", "pointer"], result: "pointer" },
  ncvisual_destroy:     { parameters: ["pointer"], result: "void" },
  ncvisual_geom:        {
    parameters: ["pointer", "pointer", "pointer", "pointer"],
    result: "i32"
  },

  // ── Input ────────────────────────────────────────────────────────────────
  notcurses_get:      { parameters: ["pointer", "pointer", "pointer"], result: "u32" },
  notcurses_mice_enable: { parameters: ["pointer", "u32"], result: "i32" },

  // ── Capabilities ─────────────────────────────────────────────────────────
  notcurses_capabilities: { parameters: ["pointer"], result: "pointer" },
  notcurses_canpixel:     { parameters: ["pointer"], result: "u32" },

  // ── Borders and lines ─────────────────────────────────────────────────────
  ncplane_box:   {
    parameters: ["pointer", "pointer", "pointer", "u32", "u32", "u32"],
    result: "i32"
  },
  ncplane_hline: { parameters: ["pointer", "pointer", "u32"], result: "i32" },
  ncplane_vline: { parameters: ["pointer", "pointer", "u32"], result: "i32" },
} as const);

export const nc = lib.symbols;
```

### 1.2  TypeScript wrapper layer

Raw FFI symbols return opaque `Deno.PointerValue`. The wrapper layer converts them to
typed handles, manages memory lifetimes, and provides an ergonomic API for the renderer.

```typescript
// src/ffi/handles.ts

/** Opaque handle to a notcurses instance. */
export type NcHandle = Deno.PointerValue & { readonly __nc: unique symbol };

/** Opaque handle to an ncplane. */
export type PlaneHandle = Deno.PointerValue & { readonly __plane: unique symbol };

/** Opaque handle to an ncvisual (image). */
export type VisualHandle = Deno.PointerValue & { readonly __visual: unique symbol };
```

```typescript
// src/ffi/api.ts — typed wrappers around raw symbols

export function ncInit(): NcHandle {
  const handle = nc.notcurses_init(null, null);
  if (!handle) throw new Error("notcurses_init failed");
  return handle as NcHandle;
}

export function ncStop(nc: NcHandle): void {
  lib.symbols.notcurses_stop(nc);
}

export function ncRender(nc: NcHandle): void {
  const result = lib.symbols.notcurses_render(nc);
  if (result < 0) throw new Error("notcurses_render failed");
}

export function ncDimensions(nc: NcHandle): { rows: number; cols: number } {
  const rows = new Int32Array(1);
  const cols = new Int32Array(1);
  lib.symbols.notcurses_stddim_yx(nc, rows, cols);
  return { rows: rows[0], cols: cols[0] };
}

export function planeCreate(
  parent: PlaneHandle,
  opts: { y: number; x: number; rows: number; cols: number }
): PlaneHandle {
  // ncplane_options struct layout (simplified)
  const nopts = new BigInt64Array(8);
  nopts[0] = BigInt(opts.y);
  nopts[1] = BigInt(opts.x);
  nopts[2] = BigInt(opts.rows);
  nopts[3] = BigInt(opts.cols);
  const handle = lib.symbols.ncplane_create(parent, nopts);
  if (!handle) throw new Error("ncplane_create failed");
  return handle as PlaneHandle;
}

export function planePutText(
  plane: PlaneHandle,
  y: number,
  x: number,
  text: string
): void {
  const encoded = new TextEncoder().encode(text + "\0");
  lib.symbols.ncplane_putstr_yx(plane, y, x, encoded);
}

export function planeSetFg(plane: PlaneHandle, r: number, g: number, b: number): void {
  lib.symbols.ncplane_set_fg_rgb8(plane, r, g, b);
}

export function planeSetBg(plane: PlaneHandle, r: number, g: number, b: number): void {
  lib.symbols.ncplane_set_bg_rgb8(plane, r, g, b);
}

// NCSTYLE_* constants from notcurses.h
export const NcStyle = {
  BOLD:      0x0001,
  ITALIC:    0x0002,
  UNDERLINE: 0x0004,
  STRUCK:    0x0010,
} as const;

export function planeSetStyles(plane: PlaneHandle, styles: number): void {
  lib.symbols.ncplane_set_styles(plane, styles);
}

export function pixelGeom(plane: PlaneHandle): {
  pxy: number; pxx: number;
  celldimy: number; celldimx: number;
} {
  const out = new Uint32Array(6);
  lib.symbols.ncplane_pixel_geom(plane, out, out.subarray(1), out.subarray(2),
    out.subarray(3), out.subarray(4), out.subarray(5));
  return { pxy: out[0], pxx: out[1], celldimy: out[2], celldimx: out[3] };
}

export function visualFromMemory(
  data: Uint8Array,
  rows: number,
  cols: number
): VisualHandle {
  const handle = lib.symbols.ncvisual_from_memory(data, BigInt(data.length), rows, cols, 0);
  if (!handle) throw new Error("ncvisual_from_memory failed");
  return handle as VisualHandle;
}

export function visualBlit(
  nc: NcHandle,
  visual: VisualHandle,
  plane: PlaneHandle
): void {
  // ncvisual_options with blitter auto-selection (Kitty, Sixel, or fallback)
  const vopts = new BigInt64Array(16);
  vopts[0] = BigInt(Deno.UnsafePointer.value(plane));
  // NCBLIT_DEFAULT lets notcurses pick the best blitter for this terminal
  vopts[4] = 0n; // blitter = NCBLIT_DEFAULT
  lib.symbols.ncvisual_blit(nc, visual, vopts);
}

export function visualDestroy(visual: VisualHandle): void {
  lib.symbols.ncvisual_destroy(visual);
}

export function canPixel(nc: NcHandle): boolean {
  return lib.symbols.notcurses_canpixel(nc) !== 0;
}
```

### 1.3  Image support is a first-class concern

`visualBlit` with `NCBLIT_DEFAULT` automatically selects the best pixel blitter for the
current terminal: Kitty Graphics Protocol, Sixel, Unicode half-blocks, or ASCII fallback.
The renderer never checks which protocol the terminal supports — Notcurses handles that
transparently.

For images fetched from the network or AT Protocol blobs (not local files), use
`ncvisual_from_memory` with the decoded image bytes. The renderer fetches the image
data in TypeScript, then passes the buffer across FFI.

```typescript
// Image rendering pipeline for AT blob refs
async function renderAtBlob(
  nc: NcHandle,
  plane: PlaneHandle,
  ref: { did: string; cid: string }
): Promise<void> {
  const bytes = await fetchAtBlob(ref.did, ref.cid); // TypeScript, gated by permission
  const decoded = await decodeImage(bytes);           // decode PNG/JPEG to RGBA pixels
  const visual = visualFromMemory(decoded.data, decoded.height, decoded.width);
  try {
    visualBlit(nc, visual, plane);
  } finally {
    visualDestroy(visual); // always free native memory
  }
}
```

---

## Part 2 — unode TUI renderer

### 2.1  Core data structures

```typescript
// src/renderer/types.ts

/** A mounted plane corresponding to a CanonicalNode in the AST. */
export interface MountedNode {
  readonly nodeKey: string;
  readonly kind: string;
  plane: PlaneHandle;
  /** Absolute position in terminal cells. */
  bounds: { y: number; x: number; rows: number; cols: number };
  children: MountedNode[];
  /** Subscription teardown for reactive nodes. */
  unsubscribe?: () => void;
}

/** Focus ring entry. */
export interface FocusEntry {
  nodeKey: string;
  plane: PlaneHandle;
  action?: ActionRef;
}
```

### 2.2  Layout engine

The layout engine translates the unode AST layout semantics into absolute terminal cell
coordinates. The terminal has no CSS, no Yoga — layout is computed in TypeScript.

```typescript
// src/renderer/layout.ts

export type Rect = { y: number; x: number; rows: number; cols: number };

/**
 * Computes absolute rects for all children given a parent rect and layout node.
 * This is a single-pass top-down layout — no reflow, no intrinsic sizing.
 * All sizing is based on terminal cell counts, not pixels.
 */
export function computeLayout(
  node: CanonicalUiNode,
  available: Rect,
  config: RendererConfig
): Map<string, Rect> {
  const rects = new Map<string, Rect>();

  switch (node.kind) {
    case "stack":
      layoutStack(node, available, rects, config);
      break;
    case "inline":
      layoutInline(node, available, rects, config);
      break;
    case "grid":
      layoutGrid(node, available, rects, config);
      break;
    case "scroll":
      layoutScroll(node, available, rects, config);
      break;
    default:
      // Leaf node — occupies its full available rect
      rects.set(node._key, available);
  }

  return rects;
}

function layoutStack(
  node: CanonicalNode<StackNode>,
  available: Rect,
  rects: Map<string, Rect>,
  config: RendererConfig
): void {
  const gap = resolveGapCells(node.gap, config);
  const childCount = node.children.length;
  const totalGap = gap * Math.max(0, childCount - 1);
  const rowsPerChild = Math.floor((available.rows - totalGap) / childCount);

  let currentY = available.y;
  for (const child of node.children) {
    const childRect = { y: currentY, x: available.x, rows: rowsPerChild, cols: available.cols };
    rects.set(child._key, childRect);
    // Recurse for container children
    const childLayout = computeLayout(child, childRect, config);
    for (const [k, v] of childLayout) rects.set(k, v);
    currentY += rowsPerChild + gap;
  }
}

function layoutGrid(
  node: CanonicalNode<GridNode>,
  available: Rect,
  rects: Map<string, Rect>,
  config: RendererConfig
): void {
  // Resolve column count from terminal width breakpoint
  const cols = resolveGridColumns(node.columns, available.cols, config.breakpoints);
  const gap = resolveGapCells(node.gap, config);
  const colWidth = Math.floor((available.cols - gap * (cols - 1)) / cols);
  const items = node.children;

  items.forEach((child, i) => {
    const col = i % cols;
    const row = Math.floor(i / cols);
    // Each grid cell height — estimated from aspect ratio or default 8 rows
    const cellRows = estimateCellRows(child, colWidth, config);
    const rect: Rect = {
      y: available.y + row * (cellRows + gap),
      x: available.x + col * (colWidth + gap),
      rows: cellRows,
      cols: colWidth,
    };
    rects.set(child._key, rect);
    const childLayout = computeLayout(child, rect, config);
    for (const [k, v] of childLayout) rects.set(k, v);
  });
}

function resolveGridColumns(
  columns: ResponsiveGridColumns | undefined,
  terminalCols: number,
  breakpoints: RendererBreakpoints
): number {
  if (!columns) return 1;
  // TUI breakpoints are in terminal columns (chars), not pixels
  if (terminalCols >= breakpoints.xl && columns.xl) return columns.xl;
  if (terminalCols >= breakpoints.lg && columns.lg) return columns.lg;
  if (terminalCols >= breakpoints.md && columns.md) return columns.md;
  if (terminalCols >= breakpoints.sm && columns.sm) return columns.sm;
  return columns.base ?? 1;
}

const GAP_CELLS: Record<string, number> = {
  none: 0, xs: 1, sm: 1, md: 2, lg: 3
};
function resolveGapCells(gap: string | undefined, _config: RendererConfig): number {
  return GAP_CELLS[gap ?? "none"] ?? 0;
}
```

### 2.3  Node painter

The painter walks the normalized AST and calls FFI functions to draw each node into its
computed rect. It maintains a map from `nodeKey` to `MountedNode` for granular updates.

```typescript
// src/renderer/painter.ts

export class NodePainter {
  private readonly mounted = new Map<string, MountedNode>();
  private readonly focusRing: FocusEntry[] = [];

  constructor(
    private readonly nc: NcHandle,
    private readonly stdplane: PlaneHandle,
    private readonly config: RendererConfig,
    private readonly theme: TuiTheme,
  ) {}

  /** Initial mount: paint the entire screen AST. */
  mount(screen: CanonicalScreen, stateStore: StateStore): void {
    const { rows, cols } = ncDimensions(this.nc);
    const rects = computeLayout(screen, { y: 0, x: 0, rows, cols }, this.config);
    this.focusRing.length = 0;

    for (const child of screen.children) {
      this.mountNode(child, rects, stateStore);
    }

    ncRender(this.nc);
  }

  /**
   * Granular patch: re-paint only the nodes that depend on a changed state path.
   * Called by the reactive update loop — never re-mounts the full screen.
   */
  patch(
    nodeKeys: readonly string[],
    screen: CanonicalScreen,
    ctx: ResolverContext
  ): void {
    for (const key of nodeKeys) {
      const mounted = this.mounted.get(key);
      if (!mounted) continue;

      const node = findNodeByKey(screen, key);
      if (!node) continue;

      // Clear the plane and re-draw only this node
      lib.symbols.ncplane_erase(mounted.plane);
      this.paintNode(node, mounted.plane, mounted.bounds, ctx);
    }

    ncRender(this.nc);
  }

  private mountNode(
    node: CanonicalUiNode,
    rects: Map<string, Rect>,
    stateStore: StateStore,
  ): void {
    const rect = rects.get(node._key);
    if (!rect) return;

    const plane = planeCreate(this.stdplane, rect);
    const mounted: MountedNode = {
      nodeKey: node._key,
      kind: node.kind,
      plane,
      bounds: rect,
      children: [],
    };
    this.mounted.set(node._key, mounted);

    // Set up reactive subscription for this node if it has bindings
    if (node._reactivity !== "static") {
      mounted.unsubscribe = stateStore.subscribePrefix(
        this.collectBindingPaths(node),
        () => this.scheduleNodePatch(node._key)
      );
    }

    this.paintNode(node, plane, rect, { state: stateStore, route: this.currentRoute, locale: this.locale });

    // Recurse into children
    if ("children" in node) {
      for (const child of (node as CanonicalNode<StackNode>).children) {
        const childMounted = this.mountNode(child, rects, stateStore);
        if (childMounted) mounted.children.push(childMounted);
      }
    }
  }

  private paintNode(
    node: CanonicalUiNode,
    plane: PlaneHandle,
    rect: Rect,
    ctx: ResolverContext
  ): void {
    switch (node.kind) {
      case "text":     return this.paintText(node, plane, rect, ctx);
      case "value":    return this.paintValue(node, plane, rect, ctx);
      case "badge":    return this.paintBadge(node, plane, rect, ctx);
      case "media":    return this.paintMedia(node, plane, rect, ctx);
      case "action":   return this.paintAction(node, plane, rect, ctx);
      case "input":    return this.paintInput(node, plane, rect, ctx);
      case "divider":  return this.paintDivider(node, plane, rect, ctx);
      case "loading":  return this.paintLoading(node, plane, rect, ctx);
      case "status":   return this.paintStatus(node, plane, rect, ctx);
      case "empty":    return this.paintEmpty(node, plane, rect, ctx);
      case "disclosure": return this.paintDisclosure(node, plane, rect, ctx);
      case "conditional": return this.paintConditional(node, plane, rect, ctx);
      case "slot":     return this.paintSlot(node, plane, rect, ctx);
      // Container nodes: no direct painting, only children (already handled by mountNode)
      case "stack": case "inline": case "grid": case "scroll":
      case "section": case "form": case "item":
      case "list": case "actions": case "menu":
        this.paintBorder(node, plane, rect);
        break;
    }
  }
```

### 2.4  AST node painting — detailed mapping

#### Text, value, badge

```typescript
  private paintText(
    node: CanonicalNode<TextNode>,
    plane: PlaneHandle,
    rect: Rect,
    ctx: ResolverContext
  ): void {
    const content = typeof node.content === "string"
      ? node.content
      : resolver.resolveString(node.content, ctx, node._key);

    const { fg, style } = this.theme.text(node.role, node.tone, node.emphasis);
    planeSetFg(plane, fg.r, fg.g, fg.b);
    planeSetStyles(plane, style);

    const truncated = node.truncate && content.length > rect.cols
      ? content.slice(0, rect.cols - 1) + "…"
      : content;

    planePutText(plane, 0, 0, truncated);
  }

  private paintBadge(
    node: CanonicalNode<BadgeNode>,
    plane: PlaneHandle,
    rect: Rect,
    ctx: ResolverContext
  ): void {
    const label = resolver.resolveString(node.label, ctx, node._key);
    const { fg, bg } = this.theme.badge(node.tone);
    planeSetFg(plane, fg.r, fg.g, fg.b);
    planeSetBg(plane, bg.r, bg.g, bg.b);
    planePutText(plane, 0, 0, ` ${label} `);
  }
```

#### Media — image-first rendering

```typescript
  private paintMedia(
    node: CanonicalNode<MediaNode>,
    plane: PlaneHandle,
    rect: Rect,
    _ctx: ResolverContext
  ): void {
    if (node.ref.type === "placeholder") {
      // Placeholder: draw a bordered box with the label
      lib.symbols.ncplane_box(plane, null, null, rect.rows - 1, rect.cols - 1, 0);
      const label = node.ref.label ?? node.alt;
      const centered = label.slice(0, rect.cols - 2);
      planePutText(plane, Math.floor(rect.rows / 2), Math.floor((rect.cols - centered.length) / 2), centered);
      return;
    }

    // Real image — enqueue async fetch, show placeholder until ready
    planePutText(plane, 0, 0, "⏳");
    this.fetchAndBlitImage(node, plane, rect);
  }

  private async fetchAndBlitImage(
    node: CanonicalNode<MediaNode>,
    plane: PlaneHandle,
    rect: Rect
  ): Promise<void> {
    try {
      let bytes: Uint8Array;
      if (node.ref.type === "url") {
        const resp = await fetch(node.ref.src);
        bytes = new Uint8Array(await resp.arrayBuffer());
      } else if (node.ref.type === "at-blob") {
        bytes = await this.atBlobResolver.fetch(node.ref.did, node.ref.cid);
      } else {
        return;
      }

      const decoded = await decodeImageToRgba(bytes);
      const visual = visualFromMemory(decoded.data, decoded.height, decoded.width);
      try {
        // Notcurses automatically picks Kitty, Sixel, or Unicode fallback
        visualBlit(this.nc, visual, plane);
        ncRender(this.nc);
      } finally {
        visualDestroy(visual);
      }
    } catch {
      // On failure, leave the placeholder
    }
  }
```

#### DisclosureNode — inline collapsible region

`DisclosureNode` is not a "toast" — it is an inline accordion that expands in place,
pushing content below it downward. The renderer manages this by:

1. Painting the trigger line (label + chevron indicator) in the disclosure plane.
2. Maintaining a child plane for the content region, initially with height 0.
3. When the bound state path changes, toggling the content plane visibility and
   triggering a layout recompute for all siblings below this node.

```typescript
  private paintDisclosure(
    node: CanonicalNode<DisclosureNode>,
    plane: PlaneHandle,
    rect: Rect,
    ctx: ResolverContext
  ): void {
    const isExpanded = ctx.state.getPrimitive(node.binding, false) as boolean;
    const label = isExpanded && node.labelExpanded
      ? resolver.resolveString(node.labelExpanded, ctx, node._key)
      : resolver.resolveString(node.label, ctx, node._key);

    const chevron = isExpanded ? "▼" : "▶";
    const { fg } = this.theme.disclosure(isExpanded);
    planeSetFg(plane, fg.r, fg.g, fg.b);
    planePutText(plane, 0, 0, `${chevron} ${label}`);

    // Content plane: only visible when expanded
    const contentKey = `${node._key}:content`;
    const contentMounted = this.mounted.get(contentKey);

    if (isExpanded) {
      if (!contentMounted) {
        // First expansion: create and mount child planes for content nodes
        const contentRect: Rect = {
          y: rect.y + 1,
          x: rect.x + 2, // indent
          rows: rect.rows - 1,
          cols: rect.cols - 2,
        };
        const contentRects = computeLayout(
          { kind: "stack", children: node.children, _key: contentKey } as any,
          contentRect,
          this.config
        );
        for (const child of node.children) {
          this.mountNode(child, contentRects, ctx.state);
        }
      } else {
        // Already mounted: make visible
        lib.symbols.ncplane_move_yx(contentMounted.plane, rect.y + 1, rect.x + 2);
      }
    } else if (contentMounted) {
      // Collapse: move plane off-screen (notcurses has no "hide" API)
      lib.symbols.ncplane_move_yx(contentMounted.plane, -1000, 0);
    }

    ncRender(this.nc);
  }
```

#### ConditionalNode — branch switching

```typescript
  private paintConditional(
    node: CanonicalNode<ConditionalNode>,
    plane: PlaneHandle,
    rect: Rect,
    ctx: ResolverContext
  ): void {
    const condition = resolver.resolveBoolean(node.condition, ctx, node._key);
    const active = condition ? node.then : node.else;
    const inactive = condition ? node.else : node.then;

    // Move inactive branch off-screen
    if (inactive) {
      const inactiveMounted = this.mounted.get((inactive as CanonicalUiNode)._key);
      if (inactiveMounted) {
        lib.symbols.ncplane_move_yx(inactiveMounted.plane, -1000, 0);
      }
    }

    // Paint active branch in this rect
    if (active) {
      this.mountNode(active as CanonicalUiNode, new Map([[
        (active as CanonicalUiNode)._key, rect
      ]]), ctx.state);
    }
  }
```

#### Action nodes — focus ring integration

```typescript
  private paintAction(
    node: CanonicalNode<ActionNode>,
    plane: PlaneHandle,
    rect: Rect,
    ctx: ResolverContext
  ): void {
    const label = resolver.resolveString(node.label, ctx, node._key);
    const disabled = resolver.resolveBoolean(node.disabled ?? false, ctx, node._key);
    const isFocused = this.focusManager.currentKey === node._key;

    const { fg, bg, style } = this.theme.action(node.intent, disabled, isFocused);
    planeSetFg(plane, fg.r, fg.g, fg.b);
    planeSetBg(plane, bg.r, bg.g, bg.b);
    planeSetStyles(plane, style);

    // Draw border for button variant
    if (node.variant !== "link" && node.variant !== "menu-item") {
      lib.symbols.ncplane_box(plane, null, null, rect.rows - 1, rect.cols - 1, 0);
    }

    planePutText(plane, Math.floor(rect.rows / 2), 2, label);

    // Register in focus ring if not disabled
    if (!disabled) {
      this.focusRing.push({ nodeKey: node._key, plane, action: node.action });
    }
  }
```

### 2.5  Reactive update loop

This is the core of the granular update system. It connects the `StateStore` subscription
model from the unode `normalize.ts` metadata to actual plane repaints.

```typescript
// src/renderer/reactive.ts

export class ReactiveLoop {
  private pendingPatches = new Set<string>();
  private rafScheduled = false;

  constructor(
    private readonly painter: NodePainter,
    private readonly resolver: DefaultExprResolver,
    private readonly stateStore: StateStore,
    private readonly screen: CanonicalScreen,
    private readonly ctx: ResolverContext,
  ) {
    // Subscribe to all reactive paths in the normalized AST
    this.subscribeAll(screen);
  }

  private subscribeAll(node: CanonicalUiNode | CanonicalScreen): void {
    if (node._reactivity !== "static") {
      // Ask the resolver which state paths this node reads
      for (const path of this.resolver.dependenciesOf(node._key)) {
        this.stateStore.subscribe(path, () => {
          this.pendingPatches.add(node._key);
          this.scheduleFlush();
        });
      }
    }

    if ("children" in node) {
      for (const child of (node as CanonicalNode<StackNode>).children) {
        this.subscribeAll(child);
      }
    }

    // Handle nodes with single child or branch children
    if (node.kind === "pressable") this.subscribeAll(node.child as CanonicalUiNode);
    if (node.kind === "conditional") {
      this.subscribeAll(node.then as CanonicalUiNode);
      if (node.else) this.subscribeAll(node.else as CanonicalUiNode);
    }
  }

  private scheduleFlush(): void {
    if (this.rafScheduled) return;
    this.rafScheduled = true;
    // Use queueMicrotask to batch multiple state changes in the same tick
    queueMicrotask(() => this.flush());
  }

  private flush(): void {
    this.rafScheduled = false;
    if (this.pendingPatches.size === 0) return;

    const keys = Array.from(this.pendingPatches);
    this.pendingPatches.clear();

    // Clear resolver tracking for affected nodes before re-evaluation
    for (const key of keys) {
      this.resolver.clearTracking(key);
    }

    // Repaint only affected nodes
    this.painter.patch(keys, this.screen, this.ctx);
  }

  teardown(): void {
    // Unsubscribe all mounted node subscriptions
    this.painter.unmountAll();
  }
}
```

### 2.6  Focus and keyboard navigation

```typescript
// src/renderer/focus.ts

export class FocusManager {
  private ring: FocusEntry[] = [];
  private currentIndex = -1;

  get currentKey(): string | undefined {
    return this.ring[this.currentIndex]?.nodeKey;
  }

  setRing(entries: FocusEntry[]): void {
    this.ring = entries;
    this.currentIndex = entries.length > 0 ? 0 : -1;
  }

  next(): FocusEntry | undefined {
    if (this.ring.length === 0) return undefined;
    this.currentIndex = (this.currentIndex + 1) % this.ring.length;
    return this.ring[this.currentIndex];
  }

  prev(): FocusEntry | undefined {
    if (this.ring.length === 0) return undefined;
    this.currentIndex = (this.currentIndex - 1 + this.ring.length) % this.ring.length;
    return this.ring[this.currentIndex];
  }

  activate(): ActionRef | undefined {
    return this.ring[this.currentIndex]?.action;
  }
}
```

### 2.7  Input event loop

The input loop runs on the main Deno thread in a `while(true)` using
`notcurses_get` with a short timeout. It is not async — it blocks for up to 16ms
waiting for input, then yields. This is the standard pattern for TUI apps.

```typescript
// src/renderer/input.ts

export async function runInputLoop(
  nc: NcHandle,
  focusManager: FocusManager,
  dispatch: (action: ActionRef) => Promise<void>,
  stateStore: StateStore,
): Promise<void> {
  const timeoutMs = new BigInt64Array([16_000_000n]); // 16ms in nanoseconds
  const inputBuf = new BigUint64Array(1);

  while (true) {
    const codepoint = lib.symbols.notcurses_get(nc, timeoutMs, inputBuf);

    if (codepoint === 0) {
      // Timeout — yield to allow other async work
      await new Promise(r => setTimeout(r, 0));
      continue;
    }

    await handleKey(codepoint, inputBuf[0], focusManager, dispatch, stateStore);
  }
}

async function handleKey(
  codepoint: number,
  modifiers: bigint,
  focus: FocusManager,
  dispatch: (action: ActionRef) => Promise<void>,
  state: StateStore,
): Promise<void> {
  // Tab / Arrow down → next focusable
  if (codepoint === 9 || codepoint === 0x40000051) { // Tab or Down arrow
    focus.next();
    return;
  }
  // Shift+Tab / Arrow up → prev focusable
  if (codepoint === 0x40000052) { // Up arrow
    focus.prev();
    return;
  }
  // Enter → activate focused element
  if (codepoint === 13) {
    const action = focus.activate();
    if (action) await dispatch(action);
    return;
  }
  // Escape → dispatch cancel or navigate back
  if (codepoint === 27) {
    await dispatch({ type: "unode.navigate", params: { mode: "back" } });
    return;
  }
}
```

---

## Part 3 — Deno host application

### 3.1  Responsibilities

The Deno host owns sandboxing, plugin lifecycle, and message routing. It does NOT own
rendering — that is the renderer's job. The separation is clean:

| Concern | Owner |
|---|---|
| Terminal drawing | TuiRenderer (Notcurses FFI) |
| Layout computation | TuiRenderer (TypeScript) |
| Reactive updates | TuiRenderer (ReactiveLoop) |
| Plugin isolation | Deno host (Worker + permissions) |
| Permission enforcement | Deno host (PermissionGuard + Worker permissions) |
| Plugin ↔ host RPC | Deno host (postMessage protocol) |
| State store | Deno host (shared, one per screen) |
| Navigation | Deno host (Navigator) |
| Action dispatch | Deno host (ActionRegistry) |

### 3.2  Plugin sandbox via Deno Workers

Each plugin runs in a dedicated Worker with OS-level permission restrictions derived
from the plugin's `PermissionProfile`. These are enforced by the Deno runtime at the
syscall level — not by JavaScript code that could be bypassed.

```typescript
// src/host/plugin-host.ts

export class DenoPluginHost {
  private readonly workers = new Map<string, Worker>();
  private readonly guards = new Map<string, PermissionGuard>();

  async register(
    plugin: PluginDefinition,
    profile: PermissionProfile,
  ): Promise<void> {
    const guard = new DefaultPermissionGuard(profile);
    this.guards.set(plugin.manifest.id, guard);

    // Build Deno Worker permissions from PermissionProfile.
    // This is OS-level enforcement — not JavaScript.
    const permissions = this.buildWorkerPermissions(profile);

    const worker = new Worker(
      new URL("./plugin-runner.ts", import.meta.url).href,
      {
        type: "module",
        deno: { permissions },
      }
    );

    worker.postMessage({
      type: "init",
      pluginId: plugin.manifest.id,
      // Plugin module URL — Worker loads it via dynamic import
      // Only the plugin's own module graph is accessible
      pluginUrl: plugin.manifest.entrypoint,
    });

    worker.addEventListener("message", (event) =>
      this.handleWorkerMessage(plugin.manifest.id, guard, event)
    );

    this.workers.set(plugin.manifest.id, worker);
  }

  private buildWorkerPermissions(
    profile: PermissionProfile
  ): Deno.PermissionOptions {
    const httpGrant = profile.grants.find(
      g => g.permission === "http.fetch" && g.granted
    );
    const sessionRead = profile.grants.find(
      g => g.permission === "storage.session.read" && g.granted
    );
    const persistRead = profile.grants.find(
      g => g.permission === "storage.persistent.read" && g.granted
    );
    const persistWrite = profile.grants.find(
      g => g.permission === "storage.persistent.write" && g.granted
    );

    return {
      // Network: only approved origins, never wildcard unless http.fetch.any granted
      net: httpGrant?.allowedOrigins?.map(o => new URL(o.origin).hostname) ?? false,

      // Filesystem: namespaced to plugin data directory only
      read: (sessionRead || persistRead)
        ? [`${DATA_DIR}/${profile.pluginId}/`]
        : false,
      write: persistWrite
        ? [`${DATA_DIR}/${profile.pluginId}/`]
        : false,

      // Everything else: always denied for plugins
      run: false,
      ffi: false,     // plugins cannot load native libraries
      env: false,     // plugins cannot read environment variables
      hrtime: false,  // plugins cannot access high-resolution timer
    };
  }

  private async handleWorkerMessage(
    pluginId: string,
    guard: PermissionGuard,
    event: MessageEvent,
  ): Promise<void> {
    const msg = event.data;

    if (msg.type === "screen") {
      // Plugin produced a CanonicalScreen — pass to renderer
      const screen = normalizeScreen(msg.screen);
      this.renderer.mount(screen, this.stateStore);
      return;
    }

    if (msg.type === "rpc") {
      const result = await this.dispatchRpc(pluginId, guard, msg);
      const worker = this.workers.get(pluginId)!;
      worker.postMessage({ type: "rpc-response", id: msg.id, result });
    }
  }

  private async dispatchRpc(
    pluginId: string,
    guard: PermissionGuard,
    msg: RpcMessage,
  ): Promise<unknown> {
    // Built-in state operations — no permission needed beyond what the
    // plugin already declared (state is per-screen, not persistent storage)
    if (msg.method === "unode.state.get") {
      return this.stateStore.get(msg.args[0] as string);
    }
    if (msg.method === "unode.state.set") {
      this.stateStore.set(msg.args[0] as string, msg.args[1] as JsonValue);
      return;
    }

    // Navigation — no permission needed (navigation is always allowed)
    if (msg.method === "unode.navigate") {
      this.navigator.navigate(msg.args[0] as string, msg.args[1]);
      return;
    }

    // HTTP — JS-level guard + OS-level Worker permission (double enforcement)
    if (msg.method.startsWith("unode.http.")) {
      guard.assert("http.fetch");
      guard.assertOrigin(msg.args[0] as string);
      return this.executeHttp(msg.method, msg.args);
    }

    // Storage — JS-level guard + OS-level Worker permission
    if (msg.method.startsWith("unode.storage.")) {
      const perm = msg.method.includes("get")
        ? "storage.session.read"
        : "storage.session.write";
      guard.assert(perm as CoreBuiltinPermission);
      return this.executeStorage(pluginId, msg.method, msg.args);
    }

    // Host domain API — checked against HostApiMeta
    const methodMeta = this.apiMeta[msg.method];
    if (!methodMeta) {
      throw new PermissionDeniedError(pluginId, `${msg.method} (not declared in api meta)`);
    }
    guard.assert(methodMeta.permission);
    return this.executeApiMethod(msg.method, msg.args);
  }
}
```

### 3.3  Plugin runner (inside Worker)

```typescript
// src/host/plugin-runner.ts — runs inside each plugin Worker

import type { PluginDefinition } from "unode";
import { DenoWorkerPluginContextAdapter } from "./worker-adapter.ts";
import { normalizeScreen } from "unode";

let pluginId: string;
let plugin: PluginDefinition;
let adapter: DenoWorkerPluginContextAdapter;

self.addEventListener("message", async (event) => {
  const msg = event.data;

  if (msg.type === "init") {
    pluginId = msg.pluginId;
    // Dynamic import — only this plugin's module graph is loaded
    // Worker permissions restrict what it can import or fetch
    const mod = await import(msg.pluginUrl);
    plugin = mod.default as PluginDefinition;
    adapter = new DenoWorkerPluginContextAdapter(pluginId);
    return;
  }

  if (msg.type === "route") {
    const matched = plugin.routes.find(r => matchPattern(r.pattern, msg.route));
    if (!matched) return;

    adapter.setRoute(msg.route);

    const data = await matched.load(adapter as any);
    const screen = matched.render(data, adapter as any);

    self.postMessage({ type: "screen", screen });
    return;
  }

  if (msg.type === "action") {
    const handler = plugin.actions?.[msg.action.type];
    if (handler) await handler(adapter as any, msg.action.params ?? {});
    return;
  }

  if (msg.type === "rpc-response") {
    adapter.resolveRpc(msg.id, msg.result);
    return;
  }
});
```

### 3.4  Deno entry point

```typescript
// src/main.ts

import { ncInit, ncStop, canPixel } from "./ffi/api.ts";
import { TuiRenderer } from "./renderer/renderer.ts";
import { DenoPluginHost } from "./host/plugin-host.ts";
import { MemoryStateStore } from "unode";
import { tuiRendererConfig } from "unode/config";
import { plugins, permissionProfiles } from "./config/plugins.ts";

// Require FFI permission — only the host process has this
// Workers never get ffi: true
const nc = ncInit();

// Detect terminal capabilities on startup
const supportsPixel = canPixel(nc);
console.error(`[mugenx] pixel graphics: ${supportsPixel ? "yes (Kitty/Sixel)" : "no (Unicode fallback)"}`);

const stateStore = new MemoryStateStore();
const renderer = new TuiRenderer(nc, stateStore, tuiRendererConfig);
const host = new DenoPluginHost(renderer, stateStore);

// Register all plugins — each gets an isolated Worker
for (const [plugin, profile] of zip(plugins, permissionProfiles)) {
  await host.register(plugin, profile);
}

// Enable mouse input
lib.symbols.notcurses_mice_enable(nc, 3); // NCMICE_BUTTON_EVENT | NCMICE_DRAG_EVENT

// Handle terminal resize
Deno.addSignalListener("SIGWINCH", () => renderer.resize());

// Navigate to initial route
host.navigate("/");

try {
  // Run input loop — blocks until user quits
  await renderer.runInputLoop();
} finally {
  ncStop(nc);
  Deno.exit(0);
}
```

### 3.5  Running with correct Deno permissions

The host process itself needs `--allow-ffi` and `--allow-env`. Workers are spawned with
restricted permissions by the host — they never inherit the host's permissions.

```bash
deno run \
  --allow-ffi=/usr/lib/libnotcurses-core.so.3 \
  --allow-env=NOTCURSES_LIB,MUGENX_DATA_DIR \
  --allow-read=/home/user/.local/share/mugenx \
  --allow-write=/home/user/.local/share/mugenx \
  --allow-net=cdn.mugenx.com,api.bsky.network \
  src/main.ts
```

Plugin Workers are spawned with a subset of these permissions derived from their
`PermissionProfile`. The host process never grants Workers `--allow-ffi` — plugins
cannot load native libraries.

---

## Implementation order

The phases below reflect dependency order. Each phase produces something runnable.

**Phase 1 — FFI bindings** (1–2 days)
Implement `notcurses.ts` and `api.ts`. Write a standalone script that inits Notcurses,
draws "Hello TUI" in color, shows an image, and exits. Validates the FFI layer works
before building anything on top.

**Phase 2 — Layout engine** (2–3 days)
Implement `layout.ts` with `stack`, `inline`, and `grid`. Write unit tests using
hardcoded rects — no Notcurses needed. The layout engine is pure TypeScript with no FFI
dependency.

**Phase 3 — Node painter, static screens** (3–4 days)
Implement `painter.ts` for all leaf nodes and static container nodes. At the end of this
phase, a hardcoded `CanonicalScreen` with text, badges, media, and actions renders
correctly in the terminal.

**Phase 4 — Reactive loop** (2–3 days)
Connect `StateStore`, `ExprResolver`, and `ReactiveLoop`. Test with a screen that has a
binding — verify that only the affected node repaints, not the full screen.

**Phase 5 — Focus and keyboard** (2 days)
Implement `FocusManager` and `input.ts`. Tab navigation, Enter to activate, Escape to
go back. Test with a screen containing multiple `ActionNode`s.

**Phase 6 — Deno host and plugin Workers** (3–4 days)
Implement `plugin-host.ts` and `plugin-runner.ts`. Load a real plugin in a Worker,
execute load/render, receive the screen, mount it. Verify that the Worker cannot make
unauthorized network requests.

**Phase 7 — DisclosureNode, ConditionalNode, SlotNode** (2 days)
Implement the composition nodes that require state interaction. These are the most
complex nodes because they affect layout dynamically.

**Phase 8 — Full AST coverage** (2–3 days)
Fill remaining nodes: `FormNode`, `InputNode`, `MenuNode`, `ListNode` with continuation,
`ScrollNode`. At the end of this phase the renderer handles the full unode AST.