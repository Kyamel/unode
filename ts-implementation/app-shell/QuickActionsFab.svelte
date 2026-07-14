<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import { commandState } from '$lib/shared/state/command';
  import { composeOpen } from '$lib/shared/state/ui';
  import { pushNativeBackOverlay } from '$lib/shared/lib/nativeBackOverlay';
  import { m } from '$lib/shared/i18n/messages';

  const pathname = $derived(page.url.pathname);

  let hidden = $state(false);
  let open = $state(false);
  let lastScrollTop = 0;

  function toggleOpen() {
    open = !open;
  }

  function openCommandPalette() {
    open = false;
    commandState.open();
  }

  function openCompose() {
    open = false;
    composeOpen.set(true);
  }

  $effect(() => {
    // Read pathname to subscribe to route changes, then reset the FAB.
    // void silences @typescript-eslint/no-unused-expressions for this
    // intentional reactive read-without-assignment pattern.
    void pathname;
    open = false;
  });

  $effect(() => {
    if (!open) return;
    return pushNativeBackOverlay(() => {
      open = false;
    });
  });

  onMount(() => {
    const scrollRoot = document.getElementById('main-scroll');
    const target = scrollRoot ?? window;
    const readTop = () => (scrollRoot ? scrollRoot.scrollTop : window.scrollY);

    lastScrollTop = readTop();

    const onScroll = () => {
      const current = readTop();
      const delta = current - lastScrollTop;
      if (current <= 4) {
        hidden = false;
      } else if (delta > 6) {
        hidden = true;
        open = false;
      } else if (delta < -6) {
        hidden = false;
      }
      lastScrollTop = current;
    };

    target.addEventListener('scroll', onScroll, { passive: true });
    return () => target.removeEventListener('scroll', onScroll);
  });
</script>

<div
  class="fixed right-[var(--space-4)] z-[var(--z-header)] origin-bottom-right transition-all duration-300 ease-out"
  class:opacity-0={hidden}
  class:scale-75={hidden}
  class:pointer-events-none={hidden}
  style="bottom: calc(var(--bottom-bar-h) + var(--space-5) + var(--safe-area-bottom));"
>
  <div class="flex flex-col items-end gap-[var(--gap-2)]">
    <button
      type="button"
      class="quick-action-btn"
      class:quick-action-btn-open={open}
      onclick={openCommandPalette}
      aria-label={m.command_open_label()}
    >
      {m.command_short_label()}
    </button>
    <button
      type="button"
      class="quick-action-btn"
      class:quick-action-btn-open={open}
      onclick={openCompose}
      aria-label={m.feed_compose_button()}
    >
      {m.post_compose_short_label()}
    </button>
    <button
      type="button"
      class="main-fab"
      class:main-fab-open={open}
      onclick={toggleOpen}
      aria-label={m.command_actions_title()}
      aria-expanded={open}
    >
      {m.action_expand_short()}
    </button>
  </div>
</div>

<style>
  .main-fab {
    display: grid;
    place-items: center;
    width: calc(var(--size-12) + var(--space-2));
    height: calc(var(--size-12) + var(--space-2));
    border-radius: 999px;
    border: var(--border-w-heavy) solid var(--color-border-strong);
    background: var(--color-surface-1);
    color: var(--color-text);
    box-shadow: var(--shadow-lg);
    font-size: var(--fs-2xl);
    font-weight: 900;
    line-height: 1;
    transition:
      transform 260ms cubic-bezier(0.22, 1, 0.36, 1),
      background-color 260ms ease,
      color 260ms ease;
    will-change: transform;
  }

  .main-fab-open {
    transform: rotate(45deg);
    background: var(--color-primary);
    color: var(--color-primary-contrast);
  }

  .quick-action-btn {
    min-width: calc(var(--size-16) + var(--space-2));
    height: calc(var(--btn-h) - var(--space-1));
    border-radius: 999px;
    border: var(--border-w-strong) solid var(--color-border-strong);
    background: var(--color-surface-1);
    color: var(--color-text);
    box-shadow: var(--shadow-sm);
    font-size: var(--fs-2xs);
    font-weight: 900;
    letter-spacing: 0.12em;
    opacity: 0;
    transform: translateY(10px) scale(0.92);
    pointer-events: none;
    transition:
      transform 260ms cubic-bezier(0.22, 1, 0.36, 1),
      opacity 220ms ease;
  }

  .quick-action-btn-open {
    opacity: 1;
    transform: translateY(0) scale(1);
    pointer-events: auto;
  }
</style>