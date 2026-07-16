<script lang="ts">
  import type { OnAction } from "unode-svelte";

  // A host-owned Svelte component. The plugin only asks for a semantic "Button"
  // via hostSlot; this app decides how it looks.
  interface Props {
    children?: unknown;
    intent?: unknown;
    action?: { t: string } | undefined;
    dispatch: OnAction;
  }
  let { children, intent, action, dispatch }: Props = $props();
</script>

<button
  class={`ds-button ds-button--${String(intent ?? "secondary")}`}
  onclick={() => action && dispatch(action)}
>
  {String(children ?? "")}
</button>
