<script lang="ts">
  import { afterNavigate, goto } from '$app/navigation';
  import { page } from '$app/state';
  import { onMount } from 'svelte';
  import { m } from '$lib/shared/i18n/messages';
  import { loadShellNavItems, type ShellNavItem } from '$lib/widgets/app-shell/navItems';

  type NavItem = { id: string; label: string; title: string; href: string; kind?: 'more' };

  let navItems = $state<NavItem[]>([]);

  const pathname = $derived(page.url.pathname);

  function mapItems(items: ShellNavItem[]): NavItem[] {
    return items.map((item) => ({
      id: item.id,
      label: item.shortLabel,
      title: item.label,
      href: item.href
    }));
  }

  async function refreshNavItems() {
    const items = await loadShellNavItems({ page, goto });
    navItems = mapItems(items);
    showOverflow = false;
  }

  const visibleItems = $derived.by(() => {
    if (navItems.length <= 5) return navItems;
    return [
      ...navItems.slice(0, 4),
      { id: 'nav-more', label: '...', title: 'More', href: '', kind: 'more' }
    ];
  });

  const overflowItems = $derived(navItems.length > 5 ? navItems.slice(4) : []);

  const activeIndex = $derived.by(() => {
    const idx = navItems.findIndex(
      (item) => pathname === item.href || pathname.startsWith(`${item.href}/`)
    );
    return idx >= 0 ? idx : 0;
  });

  const activeVisibleIndex = $derived.by(() => {
    if (visibleItems.length === navItems.length) return activeIndex;
    return activeIndex >= 4 ? visibleItems.length - 1 : activeIndex;
  });

  let hidden = $state(false);
  let isDragging = $state(false);
  let dragMoved = $state(false);
  let dragIndex = $state(0);
  let pendingIndex = $state<number | null>(null);
  let pointerId = $state<number | null>(null);
  let lastScrollTop = 0;
  let showOverflow = $state(false);

  const indicatorIndex = $derived(isDragging ? dragIndex : (pendingIndex ?? activeVisibleIndex));

  function navigateVisible(index: number) {
    const item = visibleItems[index];
    if (!item) return;
    if (item.kind === 'more') {
      showOverflow = !showOverflow;
      return;
    }
    showOverflow = false;
    pendingIndex = index;
    void goto(item.href)
      .then(() => { pendingIndex = null; })
      .catch(() => { pendingIndex = null; });
  }

  function navigateOverflow(item: NavItem) {
    showOverflow = false;
    void goto(item.href).catch(() => {});
  }

  function updateDragIndex(clientX: number, element: HTMLElement | null) {
    if (!element) return;
    const rect = element.getBoundingClientRect();
    const ratio = (clientX - rect.left) / rect.width;
    dragIndex = Math.max(0, Math.min(visibleItems.length - 1, Math.floor(ratio * visibleItems.length)));
  }

  function onPointerDown(event: PointerEvent) {
    if (event.pointerType === 'touch') return;
    if (event.pointerType === 'mouse' && event.button !== 0) return;
    pointerId = event.pointerId;
    isDragging = true;
    dragMoved = false;
    hidden = false;
    updateDragIndex(event.clientX, event.currentTarget as HTMLElement | null);
  }

  function onPointerMove(event: PointerEvent) {
    if (!isDragging || pointerId !== event.pointerId) return;
    dragMoved = true;
    updateDragIndex(event.clientX, event.currentTarget as HTMLElement | null);
  }

  function onPointerEnd(event: PointerEvent) {
    if (pointerId !== event.pointerId) return;
    if (isDragging && dragMoved) navigateVisible(dragIndex);
    pointerId = null;
    isDragging = false;
  }

  function onTouchStart(event: TouchEvent) {
    const touch = event.touches[0];
    if (!touch) return;
    pointerId = null;
    isDragging = true;
    dragMoved = false;
    hidden = false;
    updateDragIndex(touch.clientX, event.currentTarget as HTMLElement | null);
  }

  function onTouchMove(event: TouchEvent) {
    if (!isDragging || pointerId !== null) return;
    const touch = event.touches[0];
    if (!touch) return;
    dragMoved = true;
    updateDragIndex(touch.clientX, event.currentTarget as HTMLElement | null);
    event.preventDefault();
  }

  function onTouchEnd() {
    if (!isDragging || pointerId !== null) return;
    if (dragMoved) navigateVisible(dragIndex);
    isDragging = false;
  }

  function onItemClick(index: number) {
    if (dragMoved) return;
    navigateVisible(index);
  }

  onMount(() => {
    void refreshNavItems();
    afterNavigate(() => {
      pendingIndex = null;
      void refreshNavItems();
    });

    const scrollRoot = document.getElementById('main-scroll');
    const target = scrollRoot ?? window;
    const readTop = () => (scrollRoot ? scrollRoot.scrollTop : window.scrollY);

    lastScrollTop = readTop();

    const onScroll = () => {
      if (isDragging) return;
      const current = readTop();
      const delta = current - lastScrollTop;

      if (current <= 4) hidden = false;
      else if (delta > 6) { hidden = true; }
      else if (delta < -6) hidden = false;

      lastScrollTop = current;
    };

    target.addEventListener('scroll', onScroll, { passive: true });
    return () => target.removeEventListener('scroll', onScroll);
  });
</script>

<div
  class="md:hidden fixed inset-x-0 bottom-0 z-[var(--z-header)] px-[var(--space-3)] pb-[max(var(--space-2),var(--safe-area-bottom))] transition-transform duration-300 ease-out"
  class:translate-y-[120%]={hidden}
>
  <div
    role="tablist"
    aria-label={m.nav_mobile_label()}
    tabindex="-1"
    class="relative mx-auto h-[var(--bottom-bar-h)] max-w-[420px] overflow-hidden rounded-[999px]
           border-[length:var(--border-w-strong)] border-[var(--color-border-strong)]
           bg-[var(--color-surface-1)] shadow-[var(--shadow-lg)]
           mobile-nav-gesture"
    style={`--nav-index:${indicatorIndex};--nav-count:${Math.max(1, visibleItems.length)};`}
    onpointerdown={onPointerDown}
    onpointermove={onPointerMove}
    onpointerup={onPointerEnd}
    onpointercancel={onPointerEnd}
    ontouchstart={onTouchStart}
    ontouchmove={onTouchMove}
    ontouchend={onTouchEnd}
    ontouchcancel={onTouchEnd}
  >
    <div class="mobile-indicator"></div>
    <div class="grid h-full" style={`grid-template-columns:repeat(${Math.max(1, visibleItems.length)}, minmax(0, 1fr));`}>
      {#each visibleItems as item, index (item.id)}
        <button
          type="button"
          class="relative z-10 flex h-full flex-col items-center justify-center gap-0.5 transition-colors duration-300"
          class:text-[var(--color-text-inverse)]={indicatorIndex === index}
          class:text-[var(--color-text)]={indicatorIndex !== index}
          onclick={() => onItemClick(index)}
          aria-label={item.title}
        >
          <span class="text-[length:var(--fs-xs)] font-black uppercase tracking-[0.2em] leading-none">{item.label}</span>
          <span class="text-[10px] font-semibold tracking-[0.04em] leading-none">{item.title}</span>
        </button>
      {/each}
    </div>
  </div>
</div>

{#if showOverflow && overflowItems.length > 0}
  <button
    type="button"
    class="fixed inset-0 z-[var(--z-toast)] bg-[var(--color-overlay)]"
    onclick={() => { showOverflow = false; }}
    aria-label="Close menu"
  ></button>
  <div class="fixed inset-x-0 bottom-[calc(var(--bottom-bar-h)+var(--space-6))] z-[var(--z-toast)] px-[var(--space-4)]">
    <div class="mx-auto w-full max-w-[360px] space-y-[var(--gap-2)] border-[length:var(--border-w-strong)] border-[var(--color-border-strong)] bg-[var(--color-surface-1)] p-[var(--space-3)] shadow-[var(--shadow-lg)]">
      {#each overflowItems as item (item.id)}
        <button
          type="button"
          class="flex w-full items-center justify-between border-[length:var(--border-w-strong)] border-[var(--color-border-strong)] bg-[var(--color-bg)] px-[var(--space-3)] py-[var(--space-2)] text-left text-[length:var(--fs-xs)] font-black uppercase tracking-widest"
          onclick={() => navigateOverflow(item)}
        >
          <span>{item.label}</span>
          <span class="text-[length:var(--fs-2xs)] font-semibold tracking-[0.2em] text-[var(--color-text-muted)]">{item.title}</span>
        </button>
      {/each}
    </div>
  </div>
{/if}

<style>
  .mobile-indicator {
    position: absolute;
    top: 4px;
    bottom: 4px;
    left: 4px;
    width: calc((100% - 8px) / var(--nav-count));
    border-radius: 999px;
    background: var(--color-primary);
    transform: translateX(calc(var(--nav-index) * 100%));
    transition: transform 300ms cubic-bezier(0.22, 1, 0.36, 1);
    will-change: transform;
  }

  .mobile-nav-gesture {
    touch-action: none;
    user-select: none;
    -webkit-user-select: none;
  }
</style>
