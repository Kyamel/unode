import type { HostComponentProps } from "unode-react";

// A host-owned React component. The plugin never imports it. It only asks for a
// semantic "Button" via `hostSlot`, and this app decides what that looks like.
export function Button({ children, intent, dispatch, action }: HostComponentProps) {
  return (
    <button
      className={`ds-button ds-button--${String(intent ?? "secondary")}`}
      onClick={() => action && dispatch(action as { t: string })}
    >
      {String(children ?? "")}
    </button>
  );
}
