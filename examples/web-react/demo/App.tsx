// Recipes are written once, in the universal TS language. Here `action` nodes
// render as the host's native <Button> through a host slot; everything else
// falls back to the built-in DOM recipes.
import { useEffect, useState } from "react";
import {
  defineRenderer,
  h,
  hostSlot,
  ScreenStore,
  UnodeScreen,
} from "unode-react";
import type { WebRuntime } from "unode-web-core";
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
  const [state, setState] = useState<{ store: ScreenStore; runtime: WebRuntime } | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    (async () => {
      try {
        const runtime = await bootRuntime();
        setState({ store: runtime.mount(), runtime });
      } catch (e) {
        setError(String(e));
      }
    })();
  }, []);

  if (error) return <pre className="unode-error">{error}</pre>;
  if (!state) return <p>Loading unode runtime…</p>;

  return (
    <UnodeScreen
      store={state.store}
      onAction={state.runtime.onAction}
      renderer={renderer}
      components={{ Button }}
    />
  );
}
