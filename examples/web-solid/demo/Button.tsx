// A host-owned Solid component. The plugin never imports it — it asks for a
// semantic "Button" via `hostSlot`, and this app decides what it looks like.
import type { HostComponentProps } from "unode-solid";

type ButtonIntent = "primary" | "secondary" | "ghost" | "danger";

function normalizeIntent(intent: unknown): ButtonIntent {
  return intent === "primary" || intent === "secondary" || intent === "ghost" || intent === "danger"
    ? intent
    : "secondary";
}

export function Button(props: HostComponentProps) {
  const intent = () => normalizeIntent(props.intent);
  const disabled = () => props.disabled === true;

  return (
    <button
      class={`ds-button ds-button--${intent()}`}
      disabled={disabled()}
      data-intent={intent()}
      onClick={() => !disabled() && props.action && props.dispatch(props.action as { t: string })}
    >
      {String(props.children ?? "")}
    </button>
  );
}
