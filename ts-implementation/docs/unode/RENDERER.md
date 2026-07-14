# unode Renderer

## Renderer role

The renderer is not a dumb view layer. It is the platform adapter and the trust boundary.

A renderer receives a semantic AST and decides:

- how it is drawn
- how it is themed
- how users focus and navigate it
- how permissions are enforced
- how platform-native affordances map back to symbolic actions

## Renderer responsibilities

Every renderer should own:

- AST mounting
- expression resolution
- locale consumption for core i18n output
- patching or redraw strategy
- accessibility and focus behavior
- keyboard navigation behavior
- mapping actions to host/runtime handlers
- built-in capability enforcement
- storage namespacing
- event delivery
- theme and style interpretation

## Theme and style ownership

Plugins should not decide final visual presentation.

The renderer chooses:

- typography
- spacing scale
- color palette
- motion
- platform-specific affordances

That means the same AST may become:

- a rich card grid on Web
- a compact bordered list in TUI

This is a feature, not a mismatch.

## Web and TUI parity

Cross-platform does not mean pixel parity.

It means:

- the same plugin intent is preserved
- the same data and actions remain available
- keyboard/focus semantics remain coherent
- route behavior stays compatible

The Web renderer may use:

- DOM nodes
- CSS layout
- media loading
- hover states
- richer visual hierarchy

The TUI renderer may use:

- terminal cells
- text borders
- reduced media treatment
- different density defaults
- explicit focus highlighting

The AST must be shaped so both are possible.

## Keyboard and focus

Keyboard navigation is a renderer concern, but the AST should stay friendly to it.

The renderer should support:

- roving tabindex or its TUI equivalent
- arrow-key movement in navigable collections
- Enter for activation or entry
- Escape for exit or pop
- no keyboard hijacking while typing into editable inputs

The current repo already has strong Web keyboard navigation logic. That is a real asset and should inform the generic renderer contract rather than being treated as a Web-only detail.

## Sandboxing and permissions

The renderer is responsible for enforcing generic built-in permissions.

That includes capabilities like:

- HTTP
- storage
- events
- clipboard if ever added

The renderer may wrap or proxy host services before giving them to plugins.

Important rule:

- plugin manifests declare intent to use capabilities
- renderer or host decides whether they are actually granted

## Renderer contract

At a high level, a renderer needs contracts for:

- `render(node, ctx)` or equivalent mount API
- `renderMany(nodes, ctx)` if useful
- state access and state writes
- navigation integration
- permissioned built-in capabilities
- route updates

But the most important contract is conceptual:

- renderers consume semantic nodes
- renderers do not leak platform objects back into the AST

## What should not live in the renderer contract

The renderer contract should not define:

- app-specific domain APIs
- manga-specific or app-specific widgets
- direct references to the host app's component tree
- domain-specific registries layered on top of the core registries

## Practical consequence for the current codebase

The current Web renderer is implemented directly in Svelte components, which is fine as an implementation detail. The architectural gap is that some behavior still lives in those components instead of behind a cleaner renderer/runtime contract. The long-term goal is not to remove the Svelte renderer, but to make it one renderer among others.
