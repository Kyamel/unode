# unode-react

React mount target for Unode's framework-free web renderer.

`unode-react` does not own plugin loading, WASM sessions, patch planning, or the
renderer recipe language. Those live in `unode-web-core` and `unode-web-renderer`.
This package contributes only:

- `<UnodeScreen />`, which mounts a `Renderer` into a React app.
- `ReactPortalAdapter`, which fulfills `hostSlot(name)` recipes with native
  React components through React portals.

## Usage

```tsx
import { defineRenderer, h, hostSlot, UnodeScreen } from "unode-react";

const renderer = defineRenderer()
  .recipe("section", ({ title, children }) =>
    h("section", { class: "panel" }, title ? h("h2", {}, title) : null, children),
  )
  .recipe("action", ({ label, intent, action }) =>
    hostSlot("Button", { children: label, intent, action }),
  )
  .build();

export function PluginScreen({ store, runtime }) {
  return (
    <UnodeScreen
      store={store}
      onAction={runtime.onAction}
      renderer={renderer}
      components={{ Button }}
    />
  );
}
```

React and React DOM are peer dependencies so the host app controls its React
version.

## Verify

```sh
cd packages/unode-react
pnpm run typecheck
```
