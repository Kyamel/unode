// Framework-free host app. The important part: there is no React/Vue/Svelte
// here at all; the reactivity is Unode's own. The wasm host core tracks the
// plugin's state bindings and plans targeted patches; `renderer.mount` applies
// them to the DOM it created. The framework adapters only ever added a mount
// component and host-slot portals; with plain DOM recipes neither is needed.
import { defineRenderer, h } from "unode-web-renderer";
import { bootRuntime } from "./runtime";

import { buttonRecipe } from "./Button";

const renderer = defineRenderer()
  .recipe("action", buttonRecipe)
  .recipe("section", ({ title, children }) =>
    h("section", { class: "ds-card" }, title ? h("h2", {}, title) : null, children),
  )
  .build();

export async function startApp(container: HTMLElement): Promise<void> {
  try {
    const runtime = await bootRuntime();
    const store = runtime.mount();

    container.replaceChildren();
    renderer.mount(container, store, { onAction: runtime.onAction });
  } catch (error) {
    const pre = document.createElement("pre");
    pre.className = "unode-error";
    pre.textContent = String(error);
    container.replaceChildren(pre);
  }
}
