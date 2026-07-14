<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import type { Snippet } from 'svelte';
  import { contextSheetAvailable, contextSheetOpen } from '$lib/shared/state/ui';
  import { m } from '$lib/shared/i18n/messages';

  const { title = m.context_title(), children } = $props<{ title?: string; children?: Snippet }>();

  onMount(() => {
    contextSheetAvailable.set(true);
  });

  onDestroy(() => {
    contextSheetAvailable.set(false);
    contextSheetOpen.set(false);
  });
</script>

{#if $contextSheetOpen}
  <button class="fixed inset-0 z-[var(--z-drawer)] bg-[var(--color-overlay)]" onclick={() => contextSheetOpen.set(false)} aria-label={m.context_close_label()}></button>
{/if}

<div
  class={`fixed inset-x-0 bottom-0 z-[var(--z-modal)] rounded-t-[var(--radius-2xl)] border-t-[length:var(--border-w-emphasis)] border-[var(--color-border-strong)] bg-[var(--color-manga-paper)] shadow-[var(--shadow-lg)] transform transition-transform duration-200 md:hidden ${
    $contextSheetOpen ? 'translate-y-0' : 'translate-y-full'
  }`}
>
  <div class="px-[var(--space-5)] py-[var(--space-4)] border-b-[length:var(--border-w-emphasis)] border-[var(--color-border-strong)] flex items-center justify-between">
    <div>
      <div class="text-[length:var(--fs-2xs)] font-black uppercase tracking-widest text-[var(--color-text-muted)]">{m.context_label()}</div>
      <div class="text-base font-black uppercase italic">{title}</div>
    </div>
    <button class="px-[var(--space-3)] py-[var(--space-2)] border-[length:var(--border-w-strong)] border-[var(--color-border-strong)] text-[length:var(--fs-2xs)] font-black uppercase" onclick={() => contextSheetOpen.set(false)}>
      {m.action_close()}
    </button>
  </div>
  <div class="max-h-[60vh] overflow-y-auto p-[var(--space-4)] scrollbar-manga">
    {@render children?.()}
  </div>
</div>
