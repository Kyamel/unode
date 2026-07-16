# unode-svelte

Svelte mount target for Unode's framework-free web renderer.

`unode-svelte` does not own plugin loading, WASM sessions, patch planning, or the
renderer recipe language. Those live in `unode-core` and `unode-renderer`.
This package contributes only:

- `<UnodeScreen />`, which mounts a `Renderer` into a Svelte app.
- `createSveltePortal()`, which fulfills `hostSlot(name)` recipes with native
  Svelte components.

## Usage

```svelte
<script lang="ts">
  import { defineRenderer, h, hostSlot, UnodeScreen } from "unode-svelte";
  import Button from "./Button.svelte";

  const renderer = defineRenderer()
    .recipe("section", ({ title, children }) =>
      h("section", { class: "panel" }, title ? h("h2", {}, title) : null, children),
    )
    .recipe("action", ({ label, intent, action }) =>
      hostSlot("Button", { children: label, intent, action }),
    )
    .build();
</script>

<UnodeScreen {store} onAction={runtime.onAction} {renderer} components={{ Button }} />
```

Svelte is a peer dependency so the host app controls its Svelte version.

## Verify

```sh
cd packages/unode-svelte
pnpm run typecheck
```
