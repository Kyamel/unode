# Unode Roadmap

## Current State

Unode is moving from a legacy TypeScript prototype to a Rust/WASM architecture.
The Rust core is no longer only a schema port: it now includes normalization,
state, resolver tracking, patch planning, IR lowering, permissions, transport,
and a working browser host slice.

The current web proof runs:

- a Rust plugin compiled to `wasm32-unknown-unknown`;
- `unode_web_host.wasm`, built from `crates/unode-web-host`;
- JavaScript glue that wires plugin `host_call` operations;
- keyed React and Svelte adapters over `IrScreen` and `IrPatchOp`.

React and Svelte are maintained adapters, not framework requirements. Vue or a
custom adapter should consume the same IR contract.

## Near-Term Priorities

### 1. Stabilize Core Protocol Boundaries

- Document the roles of raw `ScreenNode`, `CanonicalScreen`, and `IrScreen`.
- Decide which layer is public ABI and which layers are host-internal.
- Expand golden tests for normalization, IR lowering, and patch planning.
- Keep the protocol fully serializable.

### 2. Harden The Plugin WASM ABI

- Validate required exports and ABI versions consistently.
- Improve host-call error envelopes.
- Add tests for permission-denied and missing-host-function behavior.
- Keep one plugin artifact usable by both Web and TUI packages.
- Investigate TypeScript-authored plugins as a second SDK path while preserving
  the same JSON protocol and host capability model.
- Add Component Model compatibility as a parallel loading path, starting with
  the JSON-preserving WIT contract in `wit/unode-plugin.wit`.

### 3. Normalize The Renderer Authoring Surface

The next product-shaped step is to make custom renderers easy to author. An app
team should be able to keep Unode's plugin runtime, WASM isolation, IR patches,
and reactivity model, while replacing only the visual mapping from semantic
nodes to that app's design system.

This now exists on both stacks: `defineRenderer()` on the web
(React/Svelte/Vue/Solid mount packages plus a framework-free vanilla path) and
`unode-renderer` + `unode-ratatui-renderer` on the TUI (`ratatui_renderer()`
with typed recipes, `override_render`, and `wrap`). Remaining work is the
theme/token layer and typed recipe contexts (see Decided Next Steps):

- `packages/unode-web-core` owns shared browser runtime glue: plugin WASM
  instantiation, host-session loading, plugin registry, state-write buffering,
  and action dispatch.
- `packages/unode-web-renderer` owns IR types, `ScreenStore`, patch application,
  node lookup helpers, literal/binding unwrapping, prop normalization,
  unknown-node fallback behavior, and shared renderer diagnostics.
- `packages/unode-react`, `packages/unode-svelte`, `packages/unode-vue`, and
  `packages/unode-solid` are thin mount packages for the shared renderer and
  framework-native host-slot portals.
- `examples/web-*` (react, svelte, vue, solid, vanilla) are private demos that
  wire those packages to `unode-web-core`, generated WASM artifacts, and the
  counter plugin; `examples/tui-ratatui` mirrors them on the terminal.
- Applications provide a `RendererSpec` or equivalent node map that says how
  `text`, `section`, `action`, `list`, `input`, and other semantic nodes become
  local UI components.
- The spec receives normalized props, children, action dispatch, and renderer
  context, but it does not receive plugin WASM internals or permission state.
- Default React and Svelte renderers should be examples of the same public SDK,
  not special internal paths.

This keeps the plugin protocol stable while letting each host define how plugins
look and feel inside its own application.

### 4. Package The Web Host Model

- Promote the React and Svelte slices from proofs-of-concept into reusable
  package shapes built on the renderer SDK.
- Keep framework adapters thin: IR in, patch ops applied, user actions out.
- Add documentation for embedding in React/Svelte and for writing alternate
  adapters.
- Avoid reimplementing core semantics in TypeScript.

### 5. Refine Reactivity Granularity

The current reactivity model is intentionally closer to Solid-style
targeted updates than to classic virtual DOM diffing:

- plugin UI is rendered once into a serializable AST/IR;
- `expr::binding("path")` records a dependency from state path to node key;
- `state.set("path", value)` wakes only subscribers of that path;
- patch planning re-resolves affected nodes and lowers them to compact IR patch
  ops.

This is a strong baseline, but there are known granularity limits to track:

- **Path breadth:** bindings to broad objects such as `work` will wake on writes
  to nested paths such as `work.title`. Prefer narrow bindings where possible,
  and consider typed `StatePath` helpers to make intent clearer.
- **Node-level re-resolution:** patches currently target node fields, not
  arbitrary subexpressions inside a field. Composite text or richer computed
  props may re-resolve more than the exact changed fragment.
- **Explicit bindings only:** values computed inside plugin Rust and emitted as
  literals are opaque to the host. Host-side reactivity requires dependencies to
  remain visible in the AST as expressions/bindings.
- **Indexed list paths:** paths such as `items.0.title` are useful but fragile
  when insertion or reordering changes indices. Lists need a stronger keyed
  identity story, similar in spirit to React `key`, Svelte keyed `each`, or
  Solid list helpers.
- **Stable node IDs:** precise patches depend on stable node keys. Generated
  keys can work, but interactive/stateful/plugin-extension anchors should keep
  explicit IDs.

Known framework parallels:

- Solid gets very fine updates by tracking signal reads at computation time; it
  still benefits from splitting large objects into smaller signals.
- Svelte compiles assignments into direct updates; complex object mutation still
  needs careful state shape and reassignment discipline.
- Vue proxies can track nested properties, but object/array shape and identity
  still affect update precision.
- React usually re-renders component subtrees and relies on reconciliation and
  `key`s; Unode should avoid that full-tree diff path where the binding graph can
  produce direct patches.

Future work should keep the protocol serializable while improving authoring
ergonomics around paths, computed bindings, and keyed collections.

### 6. Build The Domain Bridge Pattern

- Flesh out app-specific bridge crates such as `app-domain` and `app-sdk`.
- Add domain models, method-level permission metadata, and host-call bindings.
- Keep domain UI sugar out of `crates/unode`.
- Document plugin anchors and shell slots as app-owned extension points.

### 7. Continue The TUI Runtime

- Connect `unode-tui-runtime` session/loading helpers to a full Ratatui loop.
- Render the same IR/canonical semantics in terminal form.
- Share permission and state behavior with the web host.
- Verify that the same plugin `.wasm` can drive both environments.

## Decided Next Steps (design sessions, July 2026)

Decisions taken in design discussions but not yet implemented, in rough
priority order. Each entry records the agreed direction so implementation can
start without re-litigating the design.

### Component Model (WIT) — vertical slice DONE, web pending

The typed contract lives in `wit/unode-plugin.wit` (`unode:plugin@0.3.0`) and
is real on the TUI path: `unode-plugin-sdk` exposes
`export_plugin_component!` behind the `component` feature (same plugin
functions as `export_plugin!`), `unode-tui-runtime::ComponentTuiPlugin` loads
components via wasmtime, and a golden test proves raw ABI and component
produce identical JSON and identical host-call side effects for the same
plugin source (`plugins/counter`). Remaining:

- Web host: `jco transpile` pipeline (browsers cannot instantiate components
  natively) — the long pole.
- Componentize the remaining plugins + wire the component build into
  `build.sh`; teach the TUI loader to detect module vs component bytes.
- Generate TS manifest types from the WIT (`jco types`) so non-Rust tooling
  builds correct manifests.

### Renderer

- **Theme layer (tokens)** — the first customization layer, below
  `wrap`/`override_render`/full recipes: color/border/spacing tokens read by
  the default recipes on both stacks (`ratatui_renderer().theme(...)`, web
  `defineRenderer().theme(...)`). Most overrides are styling; today they
  require a full recipe.
- **Typed `RecipeContext` per node type (web)** — a `NodeType → props` map so
  `recipe("text", ...)` gets typed props/autocomplete instead of
  `prop("tone")` strings. Ideally the `NodeType` union (and the props map) is
  code-generated from the Rust `UiNode` enum to prevent drift.
- **Docstring pass** — `#![warn(missing_docs)]` on `unode` and
  `unode-renderer` once public items are fully documented.

### Protocol / expressions

- **Derived expressions** — `UiExpr` only has `Literal | Binding | Param`;
  there is no `$derived` equivalent (concat, formatting, arithmetic). Today
  "derived" means the plugin re-renders (`RefreshCurrentScreen`). A minimal
  computed-expression form is the most valuable `UiExpr` evolution.
- **`when` on routes/route groups** — conditional navigation entries and tabs
  (e.g. permission-gated), mirroring `SlotContributionDecl.when`.
- **i18n for manifest labels** — route `label`/`badge` are static strings or
  state bindings; message-key support (host-side `DeferredText`) is the path
  to localized navigation chrome.
- **Overlay/Layer node** — the dialog/popover/toast gap is a presentation
  layer, not content: one `Overlay` node (modal, dismissible, optional
  anchor) wrapping existing nodes; TUI renders it as a centered box.
  `ContainerRole::Dialog` exists but carries no overlay semantics.
  Calibrate by building 3–4 target screens with current nodes first.
- **Content `Tabs` node** — in-page tabs that do not change the route;
  deliberately distinct from manifest route groups.
- The node set stays **closed**: no `UiNode::Custom`. Hosts specialize by
  overriding recipes, never by inventing node types (portability guarantee).

### Plugin surfaces

- **Headless (service) plugins** — today UI is mandatory twice: `plugin_render`
  is a required export (raw ABI and WIT world) and hosts assign a fallback
  route to plugins that declare none (the counter relies on it). Decided
  direction:
  - Short term: convention *zero declared routes = no surface of its own* —
    hosts stop assigning fallback routes/nav entries (refactor: remove the
    fallback in `tui-playground::plugin_registry` and make `counter` declare
    its route); SDK sugar `export_headless_plugin! { manifest, dispatch }`
    filling `render`/`load` with empty defaults. Headless plugins act through
    slot contributions, host-dispatched actions, and (later) capabilities.
  - Long term: a second WIT world `unode:plugin/service` without the `render`
    export — the natural shape for the *provider* role in the deferred
    cross-plugin capability design (optional exports don't exist in a single
    world; profiles are separate worlds).
  - Security note: headless means no obvious user-consent moment — install
    time is the only permission gate, so host policy matters more.

- **Hover / contextual UI (LSP-style) — to analyze** — e.g. a language plugin
  contributing a dialog when hovering content another plugin rendered.
  Viability sketch: it is the slot mechanism with a *dynamic* target — instead
  of a named slot, the target is the node under the cursor. Ingredients:
  - a `render_hover(node-id, context)`-shaped export (same envelope family as
    `render_slot`), called by the host **debounced/async** — hover is
    high-frequency and must never cross the sandbox per mousemove;
  - manifest declaration of the provider (`hover for <node kinds / plugin>`),
    permission-gated like slot contributions;
  - the **Overlay node** (already on this roadmap) as the presentation layer;
  - intent, not command: TUI has no hover — hosts map it to focus + a peek
    key, exactly like tabs degrade to pages.
  Depends on Overlay and pairs naturally with capabilities/headless providers;
  analyze after those land.

### Permissions / capabilities

- **Host permission catalogs** — hosts may register known permissions so the
  loader can warn (not fail) on unknown/typo'd requests, keeping the set open.
- **Cross-plugin capabilities (deferred until a real 2-plugin use case)** —
  never direct plugin→plugin; always host-brokered, mirroring how slots broker
  UI. Shape agreed: provider declares `provides(capability("notes.search"))`
  in its manifest, consumer declares `requires`, host routes a
  `cap.invoke` host call to a new `plugin_provide` ABI export with the
  caller's identity. JSON request/response only, host timeout, no streaming.
  The manifest `requires` field already reserves the vocabulary.

### Systemic gaps to design (not yet scoped)

Identified in design review as likely-necessary for a truly generic, safe
plugin system; none has a decided shape yet.

1. **Async host calls / long operations** — the biggest architectural gap.
   Every boundary call today is synchronous request/response; `http.fetch`
   (an already-named permission) cannot work that way in a browser. Needs an
   async model — WIT async when stable, or callback-style re-dispatch
   ("host performs, then re-dispatches with the result") — plus loading-state
   conventions in the protocol.
2. **Resource limits** — CPU (fuel/epoch interruption: the TUI loader already
   carries `enable_fuel_metering`, unused), per-instance memory caps, call
   timeouts, host-call rate limiting, and render output size limits (an
   unbounded ScreenNode JSON is a DoS). Essential for "safe in any app".
3. **State namespacing** — state paths are a shared global map by convention;
   plugin A can write plugin B's `routeTabs.shipCount` today. Host-enforced
   per-plugin namespaces with explicit shared scopes (permission-gated).
4. **Crash isolation policy** — trap/panic in a plugin must never take the
   host down: quarantine, restart with backoff, and a systematic error
   surface (the playground's error panel, promoted to contract).
5. **Persistent storage** — `ctx.storage` (session/persistent, namespaced,
   quota'd) exists only in vision docs; any real app needs it.
6. **Plugin lifecycle events** — install/enable/disable/uninstall hooks and
   state/storage cleanup on uninstall; plus a state-migration story across
   plugin versions (v2 reading v1's writes).
7. **Distribution & trust** — artifact signing/content hashes in the
   manifest, provenance verification at load, and an update channel format.
8. **Host conformance kit** — generalize the raw-vs-component golden test
   into a suite any host implementation runs to prove it implements the
   contract (the "certified Unode host" story).
9. **Accessibility contract** — node semantics (labels, roles, descriptions)
   sufficient for hosts to render accessible UI; partially covered by roles,
   no systematic story.
10. **Form validation contract** — `input`/`form` nodes exist; validation
    rules, error display, and submit semantics are undefined.

### Cleanups

- **Examples share 5 copies** of `scripts/smoke.mjs` and `pkg/`; extract an
  `examples/shared/` if the duplication starts to hurt (kept copy-paste-able
  on purpose for now).
- **`unode-react` portal adapter** uses a lazily ref-initialized external
  store (lint-suppressed); migrate to `useSyncExternalStore`.

## What Should Not Change

- Plugins describe semantic UI, not DOM or terminal layout.
- Host state owns reactivity; plugin render is not called for ordinary state
  writes.
- Permissions are enforced by the host boundary.
- The core remains domain-agnostic and renderer-agnostic.
- Web embedding remains framework-agnostic.
