<script lang="ts">
  import type { OnAction } from "unode-svelte";

  // A host-owned Svelte component. The plugin only asks for a semantic "Button"
  // via hostSlot; this app decides how it looks.
  type ButtonIntent = "primary" | "secondary" | "ghost" | "danger";

  interface Props {
    children?: unknown;
    intent?: unknown;
    action?: { t: string } | undefined;
    disabled?: unknown;
    dispatch: OnAction;
  }

  let { children, intent, action, disabled, dispatch }: Props = $props();

  function normalizeIntent(value: unknown): ButtonIntent {
    return value === "primary" ||
      value === "secondary" ||
      value === "ghost" ||
      value === "danger"
      ? value
      : "secondary";
  }

  const buttonIntent = $derived(normalizeIntent(intent));
  const isDisabled = $derived(disabled === true);
</script>

<button
  class={`ds-button ds-button--${buttonIntent}`}
  data-intent={buttonIntent}
  disabled={isDisabled}
  onclick={() => !isDisabled && action && dispatch(action)}
>
  {String(children ?? "")}
</button>
