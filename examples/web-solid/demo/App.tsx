// Recipes are written once, in the universal TS language. `action` nodes
// render as the host's native <Button> through a host slot; everything else
// falls back to the built-in DOM recipes.
import { Match, Switch, createResource } from "solid-js";
import { UnodeScreen, defineRenderer, h, hostSlot } from "unode-solid";

import { Button } from "./Button";
import { bootRuntime } from "./runtime";

const renderer = defineRenderer()
  .recipe("action", ({ label, prop, action }) =>
    hostSlot("Button", { children: label, intent: prop("intent"), action }),
  )
  .recipe("section", ({ title, children }) =>
    h("section", { class: "ds-card" }, title ? h("h2", {}, title) : null, children),
  )
  .build();

export function App() {
  const [runtime] = createResource(bootRuntime);

  return (
    <Switch fallback={<p>Loading unode runtime…</p>}>
      <Match when={runtime.error}>
        <pre class="unode-error">{String(runtime.error)}</pre>
      </Match>
      <Match when={runtime()}>
        {(ready) => (
          <UnodeScreen
            store={ready().mount()}
            onAction={ready().onAction}
            renderer={renderer}
            components={{ Button }}
          />
        )}
      </Match>
    </Switch>
  );
}
