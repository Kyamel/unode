<script lang="ts">
  import SidebarActions from '$lib/widgets/app-shell/AppSidebarActions.svelte';
  import { onMount } from 'svelte';
  import { afterNavigate, goto } from '$app/navigation';
  import { page } from '$app/state';
  import { m } from '$lib/shared/i18n/messages';
  import { loadShellNavItems, type ShellNavItem } from '$lib/widgets/app-shell/navItems';

  const { onNavigate } = $props<{ onNavigate?: () => void }>();
  const APP_VERSION = __APP_VERSION__;

  let navItems = $state<ShellNavItem[]>([]);

  const pathname = $derived(page.url.pathname);

  async function refreshNavItems() {
    navItems = await loadShellNavItems({ page, goto });
  }

  let tooltipEl: HTMLElement | null = null;
  let tooltipTextEl: HTMLElement | null = null;
  let tooltipVisible = false;

  onMount(() => {
    tooltipEl = document.getElementById('global-tooltip');
    tooltipTextEl = tooltipEl?.querySelector('div:first-child') as HTMLElement;
    void refreshNavItems();
    afterNavigate(() => {
      void refreshNavItems();
    });
  });

  function showTooltip(element: HTMLElement, text: string) {
    if (!tooltipEl || !tooltipTextEl) return;
    const rect = element.getBoundingClientRect();
    tooltipTextEl.textContent = text;
    tooltipEl.style.left      = `${rect.right + 8}px`;
    tooltipEl.style.top       = `${rect.top + rect.height / 2}px`;
    tooltipEl.style.transform = 'translateY(-50%)';
    tooltipEl.style.opacity   = '1';
    tooltipVisible = true;
  }

  function hideTooltip() {
    if (!tooltipEl || !tooltipVisible) return;
    tooltipEl.style.opacity = '0';
    tooltipVisible = false;
  }
</script>

<div
  class="flex h-full w-full flex-col"
  style="padding-bottom: max(var(--safe-area-bottom), 0px);"
>
  <div class="relative [@media(max-height:480px)]:hidden">
    <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -->
    <a href="/"
      class="block cursor-pointer select-none border-b-[length:var(--border-w-emphasis)] border-[var(--color-border-strong)] px-[var(--space-2)] py-[var(--space-4)] transition-colors hover:bg-[var(--color-text)] hover:text-[var(--color-bg)]"
      onclick={() => onNavigate?.()}
      onmouseenter={(e) => showTooltip(e.currentTarget as HTMLElement, '/')}
      onmouseleave={() => hideTooltip()}
      onfocus={(e) => showTooltip(e.currentTarget as HTMLElement, '/')}
      onblur={() => hideTooltip()}
    >
      <div class="text-[length:var(--fs-2xl)] font-black uppercase italic text-[var(--color-brand-1)]">
        {m.brand_mu()}<span class="text-[var(--color-brand-2)]">{m.brand_gen()}</span>
      </div>
      <div class="mt-[var(--space-2)] text-[length:var(--fs-2xs)] font-black uppercase tracking-[0.3em] text-[var(--color-text-subtle)]">
        {m.sidebar_tagline()}
      </div>
    </a>
  </div>

  <nav class="flex-1 overflow-y-auto px-[var(--space-2)] py-[var(--space-2)]">
    {#each navItems as item (item.href)}
      {@const active = pathname === item.href || pathname.startsWith(`${item.href}/`)}
      <div class="group relative">
        <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -->
        <a href={item.href}
          class="group flex w-full items-center gap-[var(--gap-3)] border-[length:var(--border-w-strong)] text-left text-[length:var(--fs-xs)] font-black uppercase tracking-widest transition-all
                 {active
                   ? 'border-[var(--color-primary)] bg-[var(--color-primary-alpha-10)] text-[var(--color-text)]'
                   : 'border-transparent hover:border-[var(--color-border-strong)] hover:bg-[var(--color-surface-2)] focus:border-[var(--color-border-strong)] focus:bg-[var(--color-surface-2)]'
                 }"
          aria-current={active ? 'page' : undefined}
          onclick={() => onNavigate?.()}
          onmouseenter={(e) => showTooltip(e.currentTarget, item.href)}
          onmouseleave={() => hideTooltip()}
          onfocus={(e) => showTooltip(e.currentTarget, item.href)}
          onblur={() => hideTooltip()}
        >
          <span
            class="flex h-[var(--btn-h)] w-[var(--space-8)] items-center justify-center border-[length:var(--border-w-strong)] text-[length:var(--fs-2xs)]
                   {active
                     ? 'border-[var(--color-primary)] bg-[var(--color-primary)] text-[var(--color-primary-contrast)]'
                     : 'border-[var(--color-border-strong)] bg-[var(--color-surface-1)] text-[var(--color-text)]'
                   }"
          >
            {item.shortLabel}
          </span>
          <span>{item.label}</span>
        </a>
      </div>
    {/each}
  </nav>

  <div class="mt-auto px-[var(--space-2)] pb-[var(--space-2)]">
    <SidebarActions {onNavigate} />
  </div>

  <!-- Version footer — block + w-full makes it span the full sidebar width -->
  <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -->
  <a href="/about"
    class="block w-full
           border-t-[length:var(--border-w-emphasis)]
           border-[var(--color-border-strong)]
           px-[var(--space-2)]
           py-[var(--space-4)]
           text-left
           text-[length:var(--fs-2xs)]
           font-black
           uppercase
           text-[var(--color-text-subtle)]
           transition-colors
           hover:bg-[var(--color-text)]
           hover:text-[var(--color-bg)]"
    onclick={() => onNavigate?.()}
    onmouseenter={(e) => showTooltip(e.currentTarget as HTMLElement, '/about')}
    onmouseleave={() => hideTooltip()}
    onfocus={(e) => showTooltip(e.currentTarget as HTMLElement, '/about')}
    onblur={() => hideTooltip()}
  >
    {m.sidebar_version({ version: APP_VERSION })}-alpha
  </a>
</div>

<div id="global-tooltip" class="pointer-events-none fixed z-[var(--z-tooltip)] opacity-0 transition-opacity duration-200">
  <div class="whitespace-nowrap border-[length:var(--border-w-strong)] border-[var(--color-border-strong)] bg-[var(--color-surface-1)] px-[var(--space-2)] py-[var(--space-1)] text-[length:var(--fs-2xs)] font-black uppercase tracking-widest shadow-[var(--shadow-sm)]"></div>
  <div class="absolute -left-1 top-1/2 h-[var(--space-2)] w-[var(--space-2)] -translate-y-1/2 rotate-45 border-b-[length:var(--border-w-strong)] border-l-[length:var(--border-w-strong)] border-[var(--color-border-strong)] bg-[var(--color-surface-1)]"></div>
</div>
