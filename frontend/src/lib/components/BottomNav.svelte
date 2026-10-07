<script lang="ts">
  import { t } from '$lib/i18n/index.svelte';
  import { page } from '$app/state';
  import { getTabs } from '$lib/stores/sessionTabs.svelte';

  let { onOpenAll }: { onOpenAll: () => void } = $props();

  const tabs = $derived(getTabs());
  const hasActiveSessions = $derived(tabs.length > 0);

  function isActive(href: string): boolean {
    if (href === '/') {
      return page.url.pathname === '/';
    }
    return page.url.pathname.startsWith(href);
  }
</script>

<!-- Mobile Bottom Navigation Bar: sticky bottom, backdrop blur, safe-area padded -->
<nav
  aria-label={t('shell.bottomNav') || 'Navigasi Bawah'}
  class="md:hidden fixed bottom-0 left-0 right-0 z-30 bg-white/90 dark:bg-[#121212]/90 backdrop-blur-md border-t border-neutral-200 dark:border-neutral-800 pb-[env(safe-area-inset-bottom,0px)] transition-colors select-none"
>
  <div class="h-14 grid grid-cols-5 items-stretch px-1">
    <!-- 1. Hosts -->
    <a
      href="/"
      class="flex flex-col items-center justify-center gap-1 text-[10px] font-medium transition-colors {isActive('/') ? 'text-sky-600 dark:text-sky-400 font-semibold' : 'text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200'}"
    >
      <svg class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M4 17l6-5-6-5M12 19h8" />
      </svg>
      <span class="truncate max-w-[56px] leading-none">{t('nav.hosts')}</span>
    </a>

    <!-- 2. Session -->
    <a
      href="/session"
      class="relative flex flex-col items-center justify-center gap-1 text-[10px] font-medium transition-colors {isActive('/session') ? 'text-sky-600 dark:text-sky-400 font-semibold' : 'text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200'}"
    >
      <div class="relative flex items-center justify-center">
        <svg class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
        </svg>
        {#if hasActiveSessions}
          <span class="absolute -top-1 -right-1 flex h-2 w-2">
            <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
            <span class="relative inline-flex rounded-full h-2 w-2 bg-emerald-500"></span>
          </span>
        {/if}
      </div>
      <span class="truncate max-w-[56px] leading-none">{t('shell.session') || 'Session'}</span>
    </a>

    <!-- 3. Snippets -->
    <a
      href="/snippets"
      class="flex flex-col items-center justify-center gap-1 text-[10px] font-medium transition-colors {isActive('/snippets') ? 'text-sky-600 dark:text-sky-400 font-semibold' : 'text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200'}"
    >
      <svg class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4" />
      </svg>
      <span class="truncate max-w-[56px] leading-none">{t('nav.snippets')}</span>
    </a>

    <!-- 4. SFTP -->
    <a
      href="/sftp"
      class="flex flex-col items-center justify-center gap-1 text-[10px] font-medium transition-colors {isActive('/sftp') ? 'text-sky-600 dark:text-sky-400 font-semibold' : 'text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200'}"
    >
      <svg class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
      </svg>
      <span class="truncate max-w-[56px] leading-none">{t('nav.sftp')}</span>
    </a>

    <!-- 5. Semua (Launcher Modal Sheet) -->
    <button
      type="button"
      onclick={onOpenAll}
      class="flex flex-col items-center justify-center gap-1 text-[10px] font-medium text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200 transition-colors cursor-pointer"
      aria-label="Buka Semua Menu"
    >
      <svg class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z" />
      </svg>
      <span class="truncate max-w-[56px] leading-none">{t('shell.allMenus') || 'Semua'}</span>
    </button>
  </div>
</nav>
