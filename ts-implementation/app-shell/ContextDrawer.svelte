<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import type { Snippet } from 'svelte';
  import { contextDrawerAvailable, contextDrawerOpen } from '$lib/shared/state/ui';
  import { m } from '$lib/shared/i18n/messages';

  const { title = m.context_title(), children } = $props<{ title?: string; children?: Snippet }>();

  onMount(() => {
    contextDrawerAvailable.set(true);
  });

  onDestroy(() => {
    contextDrawerAvailable.set(false);
    contextDrawerOpen.set(false);
  });
</script>

{#if $contextDrawerOpen}
  <button class="fixed inset-0 z-[var(--z-drawer)] bg-[var(--color-overlay-strong)]" onclick={() => contextDrawerOpen.set(false)} aria-label={m.context_close_label()}></button>
{/if}

<aside
  class={`fixed left-0 bottom-0 top-[var(--header-safe-h)] z-[var(--z-modal)] w-[var(--sidebar-w)] border-r-[length:var(--border-w-emphasis)] border-[var(--color-border-strong)] bg-[var(--color-manga-paper)] transform transition-transform duration-200 md:hidden ${$contextDrawerOpen ? 'translate-x-0' : '-translate-x-full'}`}
>
  <div class="flex h-full flex-col">
    <div class="px-[var(--space-5)] py-[var(--space-4)] border-b-[length:var(--border-w-emphasis)] border-[var(--color-border-strong)]">
      <div class="text-[length:var(--fs-xs)] font-black uppercase tracking-widest text-[var(--color-text-muted)]">{m.context_drawer_label()}</div>
      <div class="text-[length:var(--fs-lg)] font-black uppercase italic">{title}</div>
    </div>
    <div class="flex-1 overflow-y-auto p-[var(--space-4)] scrollbar-manga">
      {@render children?.()}
    </div>
  </div>
</aside>
