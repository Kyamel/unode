<script lang="ts">
  import ThemeLanguageToggle from '$lib/shared/ui/ThemeLanguageToggle.svelte';
  import Tooltip from '$lib/shared/ui/Tooltip.svelte';
  import { m } from '$lib/shared/i18n/messages';

  const { onNavigate = null, className = '' } = $props<{
    onNavigate?: () => void;
    className?: string;
  }>();

  const baseButton =
    'h-[var(--btn-h)] w-[var(--space-8)] border-[length:var(--border-w-heavy)] border-[var(--color-border-strong)] bg-[var(--color-surface-1)] hover:bg-[var(--color-primary)] hover:text-[var(--color-primary-contrast)] transition-colors shadow-[var(--shadow-sm)] flex items-center justify-center';

</script>

<div class={`grid grid-cols-4 gap-[var(--gap-2)] ${className}`.trim()}>
  <Tooltip label={m.sidebar_tooltip_profile()} position="top">
    <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -->
    <a href="/app/profile"
      class={baseButton}
      aria-label={m.sidebar_label_profile()}
      onclick={() => onNavigate?.()}
    >
      <svg viewBox="0 0 24 24" class="h-[var(--space-5)] w-[var(--space-5)]" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
        <path d="M20 21a8 8 0 10-16 0" />
        <circle cx="12" cy="8" r="4" />
      </svg>
    </a>
  </Tooltip>

  <Tooltip label={m.sidebar_tooltip_settings()} position="top">
    <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -->
    <a href="/app/settings"
      class={baseButton}
      aria-label={m.sidebar_label_settings()}
      onclick={() => onNavigate?.()}
    >
      <svg viewBox="0 0 24 24" class="h-[var(--space-5)] w-[var(--space-5)]" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
        <circle cx="12" cy="12" r="3" />
        <path d="M19.4 15a7.4 7.4 0 000-6l2-1.5-2-3.5-2.4 1a7.9 7.9 0 00-5.2-3l-.4-2.6h-4l-.4 2.6a7.9 7.9 0 00-5.2 3l-2.4-1-2 3.5L4.6 9a7.4 7.4 0 000 6L2.6 16.5l2 3.5 2.4-1a7.9 7.9 0 005.2 3l.4 2.6h4l.4-2.6a7.9 7.9 0 005.2-3l2.4 1 2-3.5z" />
      </svg>
    </a>
  </Tooltip>
  <ThemeLanguageToggle buttonClass={baseButton} />
</div>
