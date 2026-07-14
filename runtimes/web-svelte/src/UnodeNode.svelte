<script lang="ts">
  import { createSubscriber } from "svelte/reactivity";
  import { literalOf, nodeKey, type IrNode } from "./ir";
  import type { OnAction } from "./renderer";
  import type { ScreenStore } from "./store";
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

  let replacement = $derived.by(() => {
    subscribeToNode();
    return key ? store.replacementOf(key) : undefined;
  });
  let childrenOverride = $derived.by(() => {
    subscribeToNode();
    return key ? store.childrenOverrideOf(key) : undefined;
  });
  let children = $derived(childrenOverride ?? node.c ?? []);
  let p = $derived.by(() => {
    subscribeToNode();
    return key ? store.propsOf(key) : node.p;
  });

  function textValue(value: unknown): string {
    return String(literalOf(value) ?? "");
  }

  function actionValue(value: unknown): { t: string; p?: Record<string, unknown> } | undefined {
    if (value && typeof value === "object" && "t" in value) {
      return value as { t: string; p?: Record<string, unknown> };
    }
    return undefined;
  }
</script>

{#if replacement && nodeKey(replacement) !== key}
  <UnodeNode node={replacement} {store} {onAction} />
{:else if node.t === "text"}
  <p class={`unode-text unode-text--${String(p.role ?? "body")}`}>
    {textValue(p.content)}
  </p>
{:else if node.t === "actions"}
  <div class="unode-actions">
    {#each children as child (nodeKey(child))}
      <UnodeNode node={child} {store} {onAction} />
    {/each}
  </div>
{:else if node.t === "action"}
  {@const action = actionValue(p.do)}
  <button
    class={`unode-action unode-action--${String(p.intent ?? "secondary")}`}
    disabled={Boolean(literalOf(p.dis))}
    onclick={() => action && onAction(action)}
  >
    {textValue(p.label)}
  </button>
{:else if node.t === "stack"}
  <div class="unode-stack">
    {#each children as child (nodeKey(child))}
      <UnodeNode node={child} {store} {onAction} />
    {/each}
  </div>
{:else if node.t === "inline"}
  <div class="unode-inline">
    {#each children as child (nodeKey(child))}
      <UnodeNode node={child} {store} {onAction} />
    {/each}
  </div>
{:else if node.t === "section"}
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
