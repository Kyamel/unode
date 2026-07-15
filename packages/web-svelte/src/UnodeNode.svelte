<script lang="ts">
  import { createSubscriber } from "svelte/reactivity";
  import { nodeKey, type ActionRef, type IrNode } from "unode-renderer";
  import type { OnAction } from "./renderer";
  import type { ScreenStore } from "unode-renderer";
  import UnodeNode from "./UnodeNode.svelte";

  interface Props {
    node: IrNode;
    store: ScreenStore;
    onAction: OnAction;
  }

  let { node, store, onAction }: Props = $props();
  let key = $derived(nodeKey(node));
  let subscribeToNode = $derived(
    createSubscriber((update) => (key ? store.subscribe(key, update) : undefined)),
  );

  let snapshot = $derived.by(() => {
    subscribeToNode();
    return store.snapshotOf(node);
  });
  let replacement = $derived(snapshot.replacement);
  let children = $derived(snapshot.children);
  let p = $derived(snapshot.props);
  let type = $derived(snapshot.type);

  function textValue(value: unknown): string {
    return String(value ?? "");
  }

  function actionValue(value: unknown): ActionRef | undefined {
    if (value && typeof value === "object" && "t" in value) {
      return value as ActionRef;
    }
    return undefined;
  }
</script>

{#if replacement && nodeKey(replacement) !== key}
  <UnodeNode node={replacement} {store} {onAction} />
{:else if type === "text"}
  <p class={`unode-text unode-text--${String(p.role ?? "body")}`}>
    {textValue(p.content)}
  </p>
{:else if type === "actions"}
  <div class="unode-actions">
    {#each children as child (nodeKey(child))}
      <UnodeNode node={child} {store} {onAction} />
    {/each}
  </div>
{:else if type === "action"}
  {@const action = actionValue(p.action)}
  <button
    class={`unode-action unode-action--${String(p.intent ?? "secondary")}`}
    disabled={Boolean(p.disabled)}
    onclick={() => action && onAction(action)}
  >
    {textValue(p.label)}
  </button>
{:else if type === "stack"}
  <div class="unode-stack">
    {#each children as child (nodeKey(child))}
      <UnodeNode node={child} {store} {onAction} />
    {/each}
  </div>
{:else if type === "inline"}
  <div class="unode-inline">
    {#each children as child (nodeKey(child))}
      <UnodeNode node={child} {store} {onAction} />
    {/each}
  </div>
{:else if type === "section"}
  <section class="unode-section">
    {#each children as child (nodeKey(child))}
      <UnodeNode node={child} {store} {onAction} />
    {/each}
  </section>
{:else}
  {#each children as child (nodeKey(child))}
    <UnodeNode node={child} {store} {onAction} />
  {/each}
{/if}
