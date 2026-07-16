// Recipes are written once, in the universal TS language. `action` nodes
// render as the host's native <Button> through a host slot; everything else
// falls back to the built-in DOM recipes.
import { defineComponent, h as vueH, onMounted, shallowRef } from "vue";
import { UnodeScreen, defineRenderer, h, hostSlot } from "unode-vue";
import type { WebRuntime } from "unode-web-core";

import Button from "./Button.vue";
import { bootRuntime } from "./runtime";

const renderer = defineRenderer()
  .recipe("action", ({ label, prop, action }) =>
    hostSlot("Button", { children: label, intent: prop("intent"), action }),
  )
  .recipe("section", ({ title, children }) =>
    h("section", { class: "ds-card" }, title ? h("h2", {}, title) : null, children),
  )
  .build();

export const App = defineComponent({
  name: "App",
  setup() {
    const runtime = shallowRef<WebRuntime | null>(null);
    const error = shallowRef<string | null>(null);

    onMounted(async () => {
      try {
        runtime.value = await bootRuntime();
      } catch (cause) {
        error.value = String(cause);
      }
    });

    return () => {
      if (error.value) return vueH("pre", { class: "unode-error" }, error.value);
      if (!runtime.value) return vueH("p", {}, "Loading unode runtime…");
      return vueH(UnodeScreen, {
        store: runtime.value.mount(),
        onAction: runtime.value.onAction,
        renderer,
        components: { Button },
      });
    };
  },
});
