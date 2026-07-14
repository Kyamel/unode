<script lang="ts">
  import SidebarContent from '$lib/widgets/app-shell/AppSidebarContent.svelte';
  import { m } from '$lib/shared/i18n/messages';

  const { open = false, onClose = null } = $props<{
    open?: boolean;
    onClose?: () => void;
  }>();

  const baseAside =
    'fixed left-0 top-[var(--header-safe-h)] z-[var(--z-header)] h-[calc(100vh-var(--header-safe-h))] w-[var(--sidebar-w)] border-r-[length:var(--border-w-emphasis)] border-[var(--color-border-strong)] bg-[var(--color-surface-1)]';
</script>

{#if open}
  <button
    class="md:hidden fixed inset-0 bg-[var(--color-overlay-strong)] z-[var(--z-drawer)]"
    title={m.nav_close_menu()}
    onclick={() => onClose?.()}
  ></button>
{/if}

<!-- Desktop -->
<aside class={`${baseAside} hidden md:flex`}>
  <SidebarContent onNavigate={onClose} />
</aside>

<!-- Mobile Drawer -->
<aside
  class={`${baseAside} md:hidden transform transition-transform duration-200 ${open ? 'translate-x-0' : '-translate-x-full'}`}
>
  <SidebarContent onNavigate={onClose} />
</aside>
