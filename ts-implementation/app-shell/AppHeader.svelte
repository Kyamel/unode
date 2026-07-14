<script lang="ts">
  import { page } from '$app/state';
  import { commandState } from '$lib/shared/state/command';
  import { pushNativeBackOverlay } from '$lib/shared/lib/nativeBackOverlay';
  import { m } from '$lib/shared/i18n/messages';

  type Props = {
    onToggleSidebar?: () => void;
    menuOpen?: boolean;
  };

  type HeaderTitle =
    | 'welcome'
    | 'network'
    | 'manga'
    | 'universe'
    | 'groups'
    | 'explore'
    | 'notifications'
    | 'search'
    | 'default';

  type HeaderContext = {
    title: HeaderTitle;
    subtitle?: string;
    placeholder: string;
  };

  const { onToggleSidebar, menuOpen = false }: Props = $props();

  let searchQuery = $state('');
  let notificationsOpen = $state(false);
  const pathname = $derived(page.url.pathname);

  const contexts: { match: RegExp; build: () => HeaderContext }[] = [
    {
      match: /^\/app\/?$/,
      build: () => ({
        title: 'welcome',
        subtitle: m.header_subtitle_feed(),
        placeholder: m.header_placeholder_archive()
      })
    },
    {
      match: /^\/app\/network(\/.*)?$/,
      build: () => ({
        title: 'network',
        subtitle: m.header_network_subtitle(),
        placeholder: m.header_network_placeholder()
      })
    },
    {
      match: /^\/app\/manga(\/.*)?$/,
      build: () => ({
        title: 'manga',
        subtitle: m.header_manga_subtitle(),
        placeholder: m.header_manga_placeholder()
      })
    },
    {
      match: /^\/app\/universe(\/.*)?$/,
      build: () => ({
        title: 'universe',
        subtitle: m.header_universe_subtitle(),
        placeholder: m.header_universe_placeholder()
      })
    },
    {
      match: /^\/app\/groups(\/.*)?$/,
      build: () => ({
        title: 'groups',
        subtitle: m.header_groups_subtitle(),
        placeholder: m.header_groups_placeholder()
      })
    },
    {
      match: /^\/app\/explore(\/.*)?$/,
      build: () => ({
        title: 'explore',
        subtitle: m.header_encyclopedia_subtitle(),
        placeholder: m.header_encyclopedia_placeholder()
      })
    },
    {
      match: /^\/app\/notifications(\/.*)?$/,
      build: () => ({
        title: 'notifications',
        subtitle: m.header_notifications_subtitle(),
        placeholder: m.header_notifications_placeholder()
      })
    },
    {
      match: /^\/app\/search(\/.*)?$/,
      build: () => ({
        title: 'search',
        subtitle: m.header_search_subtitle(),
        placeholder: m.header_search_placeholder()
      })
    }
  ];

  function resolveContext(currentPathname: string): HeaderContext {
    for (const entry of contexts) {
      if (entry.match.test(currentPathname)) return entry.build();
    }

    return {
      title: 'default',
      subtitle: m.header_default_subtitle(),
      placeholder: m.header_placeholder_archive()
    };
  }

  const context = $derived(resolveContext(pathname));

  function handleSearchSubmit(event: SubmitEvent) {
    if (searchQuery.trim()) return;
    event.preventDefault();
  }

  //function toggleNotifications() {
  //  notificationsOpen = !notificationsOpen;
  //}

  //function closeNotifications() {
  //  notificationsOpen = false;
  //}

  $effect(() => {
    if (!notificationsOpen) return;
    return pushNativeBackOverlay(() => {
      notificationsOpen = false;
    });
  });
</script>

<header
  class="fixed inset-x-0 top-0 z-[var(--z-header)] border-b-[length:var(--border-w-emphasis)] border-[var(--color-border-strong)] bg-[var(--color-surface-1)] py-[var(--space-1)]"
  style="padding-top: calc(var(--safe-area-top) + var(--space-1));"
>
  <div class="flex items-center justify-between gap-[var(--gap-2)] px-[var(--space-2)] md:px-[var(--space-2)]">

    <!-- MENU BUTTON -->
    <button
      class="md:hidden flex h-[var(--btn-h)] w-[var(--btn-h)] items-center justify-center border-[length:var(--border-w-strong)] border-[var(--color-border-strong)] bg-[var(--color-surface-1)] shadow-[var(--shadow-sm)] transition-colors hover:bg-[var(--color-primary)] hover:text-[var(--color-primary-contrast)]"
      aria-label={m.nav_open_menu()}
      onclick={() => onToggleSidebar?.()}
    >
      {#if menuOpen}
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2.5">
          <line x1="2" y1="2" x2="14" y2="14"/>
          <line x1="14" y1="2" x2="2" y2="14"/>
        </svg>
      {:else}
        <svg width="18" height="18" viewBox="0 0 18 18" fill="none" stroke="currentColor" stroke-width="2.5">
          <line x1="2" y1="4" x2="16" y2="4"/>
          <line x1="2" y1="9" x2="16" y2="9"/>
          <line x1="2" y1="14" x2="16" y2="14"/>
        </svg>
      {/if}
    </button>

    <!-- TITLE -->
    <div class="hidden h-[var(--btn-h)] min-w-[400px] max-w-[400px] lg:block">
      <h2 class="truncate text-[length:var(--fs-lg)] leading-none font-black uppercase italic tracking-tight">
        <span class="text-[var(--color-primary)]">/</span>

        {#if context.title === 'welcome'}
          {m.header_welcome_prefix()}
          <span class="text-[var(--color-brand-1)]">{m.brand_mu()}</span
          ><span class="text-[var(--color-brand-2)]">{m.brand_gen()}</span>

        {:else if context.title === 'network'}
          {m.header_network_title()}

        {:else if context.title === 'manga'}
          {m.header_manga_title()}

        {:else if context.title === 'universe'}
          {m.header_universe_title()}

        {:else if context.title === 'groups'}
          {m.header_groups_title()}

        {:else if context.title === 'explore'}
          {m.header_encyclopedia_title()}

        {:else if context.title === 'notifications'}
          {m.header_notifications_title()}

        {:else if context.title === 'search'}
          {m.header_search_title()}

        {:else}
          {m.header_default_title()}
        {/if}
      </h2>

      {#if context.subtitle}
        <div class="mt-[var(--space-1)] flex items-center gap-[var(--gap-2)]">
          <div class="flex gap-0.5">
            <div class="h-1.5 w-1.5 bg-[var(--color-primary)]"></div>
            <div class="h-1.5 w-1.5 bg-manga-blue-20"></div>
            <div class="h-1.5 w-1.5 bg-manga-blue-10"></div>
          </div>

          <span class="text-[length:var(--fs-2xs)] font-black uppercase tracking-[0.2em] text-[var(--color-text-muted)]">
            {context.subtitle}
          </span>
        </div>
      {/if}
    </div>

    <!-- SEARCH -->
    <form
      class="relative flex min-w-0 flex-1 items-center"
      method="GET"
      action="/app/search"
      onsubmit={handleSearchSubmit}
    >
      <button
        type="submit"
        class="absolute left-[var(--space-1)] px-[var(--space-2)] py-[var(--space-1)] text-[length:var(--fs-2xs)] font-black uppercase transition-colors hover:bg-[var(--color-primary)] hover:text-[var(--color-primary-contrast)]"
        aria-label={m.action_search()}
      >
        🔍
      </button>

      <input
        name="query"
        bind:value={searchQuery}
        placeholder={context.placeholder}
        class="h-[var(--input-h)] w-full border-[length:var(--border-w-strong)] border-[var(--color-border-strong)] bg-[var(--color-surface-1)] pl-[var(--size-12)] pr-[var(--size-20)] text-[length:var(--fs-2xs)] font-black uppercase focus:outline-none focus:ring-4 focus:ring-[var(--color-focus-ring)]"
      />

      <button
        type="button"
        class="absolute right-[var(--space-8)] px-[var(--space-2)] py-[var(--space-1)] text-[length:var(--fs-2xs)] font-black uppercase hover:bg-[var(--color-primary)] hover:text-[var(--color-primary-contrast)]"
        onclick={() => commandState.open()}
      >
        {m.command_shortcut()}
      </button>
    </form>

    <!-- NOTIFICATIONS
    <div class="relative">
      <button
        class="relative flex h-[var(--btn-h)] w-[var(--btn-h)] items-center justify-center border-[length:var(--border-w-strong)] border-[var(--color-border-strong)] bg-[var(--color-surface-1)] shadow-[var(--shadow-sm)] hover:bg-[var(--color-primary)] hover:text-[var(--color-primary-contrast)]"
        aria-expanded={notificationsOpen}
        onclick={toggleNotifications}
      >
        🔔
        <span class="absolute right-0 top-0 h-[var(--space-2)] w-[var(--space-2)] rounded-full border border-[var(--color-border-strong)] bg-[var(--color-text-inverse)]"></span>
      </button>
    </div>
    -->
  </div>
</header>
