import type { HostComponentProps } from "unode-react";

type ButtonIntent = "primary" | "secondary" | "ghost" | "danger";

function normalizeIntent(intent: unknown): ButtonIntent {
  return intent === "primary" ||
    intent === "secondary" ||
    intent === "ghost" ||
    intent === "danger"
    ? intent
    : "secondary";
}

// A host-owned React component. The plugin never imports it. It only asks for a
// semantic "Button" via `hostSlot`, and this app decides what that looks like.
export function Button({
  children,
  intent,
  dispatch,
  action,
  disabled,
}: HostComponentProps) {
  const buttonIntent = normalizeIntent(intent);
  const isDisabled = disabled === true;

  return (
    <button
      className={`ds-button ds-button--${buttonIntent}`}
      disabled={isDisabled}
      data-intent={buttonIntent}
      onClick={() => !isDisabled && action && dispatch(action as { t: string })}
    >
      {String(children ?? "")}
    </button>
  );
}
