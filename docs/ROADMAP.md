# unode Roadmap

## Current state

The TypeScript implementation is feature-complete for the web renderer:
- Canonical AST with normalization (`_reactivity`, `_subtreeReactivity`)
- `MemoryStateStore` with path subscriptions
- `ExprResolver` interface (declared, not yet wired to renderer)
- Plugin runtime with route registry, action registry, i18n
- Svelte web renderer consuming canonical AST

**Known issues in the TypeScript implementation:**

1. **Global state invalidation** — `rendererStateRevision` invalidates all
   components on any state write. Fixed in the Rust implementation by design
   (per-path subscriptions from the start).

2. **Screen resolution after mount** — `ScreenHost` resolves screens in
   `onMount`, so `preloadData` on hover does nothing for plugin screens.
   Fixed in the Rust implementation via `+page.ts load()`.

3. **Plugin activation re-runs on every navigation** — `ensurePluginsActivated`
   re-fetches the plugin registry on each call. Fixed by session-level caching.

4. **`ExprResolver` never connected** — the tracking system exists but the
   renderer uses `rendererStateRevision` instead. Fixed in the Rust
   implementation where the resolver is connected from day one.

---

## Migration phases

### Phase 0 — Fix TypeScript renderer (parallel to Rust work)

Before or alongside the Rust migration, fix the known issues in the existing
TypeScript renderer so the product is not blocked:

1. Cache `ensurePluginsActivated` in session memory — one-day fix
2. Move screen resolution to `+page.ts load()` — two-day fix
3. Replace `rendererStateRevision` with `SvelteStateAdapter` per-path stores

These fixes are described in detail in `RENDERER.md`. They are independent of
the Rust migration and should ship first.

### Phase 1 — unode in Rust (Weeks 1–2)

- AST types with Serde
- `MemoryStateStore`
- `ExprResolver` with tracking
- `normalizeScreen` with defaults, `_reactivity`, id validation
- `trackReactiveBindings`
- Transport layer (JSON envelope)
- Unit tests against TypeScript normalize spec to verify parity

**Success criterion:** Rust normalize produces identical JSON to TypeScript normalize
for the same input. Verified by a test that runs both and diffs the output.

### Phase 2 — unode-plugin-sdk in Rust (Week 3)

- DSL builders for all node types
- `PluginContext` wrapping host function calls
- WASM export boilerplate (`plugin_manifest`, `plugin_load`, `plugin_render`, `plugin_dispatch`)
- Memory allocation protocol (`unode_alloc`, `unode_dealloc`)
- One complete plugin rewritten in Rust as proof

**Success criterion:** A plugin written in Rust compiles to `.wasm` and its
`plugin_render()` output, when deserialized and normalized, matches the expected
`CanonicalScreen` structure.

### Phase 3 — mugen-bridge in Rust (Week 4)

- Domain models (`WorkSummary`, `ChapterSummary`, etc.)
- `CatalogApi`, `LibraryApi`, `ReaderApi` traits
- Host function registration for both web and TUI renderers
- Permission metadata (`HOST_FN_META`)
- Locale provider implementation
- Domain sugar (`workBanner`, `chapterList`)

**Success criterion:** A Rust plugin using `mugen-bridge` compiles to WASM, and
all domain API calls succeed when the WASM is instantiated with real host function
implementations.

### Phase 4 — Web renderer WASM integration (Week 5)

- `unode-web-runtime` crate with `wasm-bindgen` entry points
- Replace TypeScript runtime call with WASM instantiation in `PluginScreenHost`
- Implement host functions as JS closures in TypeScript
- Implement `SvelteStateAdapter` for per-path reactivity
- Move screen resolution to `+page.ts load()`

**Success criterion:** All existing plugin screens render correctly using the Rust
WASM backend. TypeScript runtime is removed from the active path.

### Phase 5 — TUI renderer (Weeks 6–8)

See `IMPLEMENTATION.md` Phase 5 for detailed steps.

**Success criterion:** A plugin that renders correctly on Web also renders
correctly in the terminal, from the same `.wasm` file.

---

## What changes for plugin authors

**Nothing visible.** Plugins are written in Rust against `unode-plugin-sdk`.
The SDK API is designed to match the ergonomics of the TypeScript DSL. The
same concepts exist: `ui.stack()`, `ui.text()`, `ctx.state.set()`,
`ctx.api::<CatalogApi>()`.

The build step changes: instead of `tsc`, plugin authors run
`cargo build --target wasm32-unknown-unknown`. The CLI toolchain handles this.

---

## What does NOT change

- The canonical AST schema — JSON format is identical
- Plugin permission declaration and approval flow
- The app bridge concept (domain APIs, permission metadata)
- i18n model (plugin-owned catalogs, `ctx.locale()` for current locale)
- The two-mode reactivity model (local SPA-like + route-driven)
- Slot system (plugin-to-plugin UI injection)

---

## Compatibility notes

During the migration, the TypeScript and Rust implementations will coexist:

- TypeScript plugins continue to work via the Javy/QuickJS WASM model
  (TypeScript compiled to WASM via esbuild + Javy)
- Rust plugins compile to native WASM
- Both run on the same Wasmtime/WebAssembly runtime
- The host cannot distinguish between them — both export the same interface

This means scan groups can continue writing TypeScript plugins while the
Rust infrastructure is being developed. The migration is opt-in per plugin,
not a flag day.
