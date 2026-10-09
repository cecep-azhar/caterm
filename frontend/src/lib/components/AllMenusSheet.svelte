<script lang="ts">
  import { t } from '$lib/i18n/index.svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';

  let {
    open = $bindable(false),
    onClose
  }: {
    open: boolean;
    onClose: () => void;
  } = $props();

  let searchQuery = $state('');

  interface MenuItem {
    id: string;
    label: string;
    href: string;
    icon: string;
    color: string;
    category?: string;
  }

  // 16 modules listed in specification:
  // Hosts, Session, SFTP, Snippets, Prompt Studio, Monitoring, Tasks, Port Fwd,
  // Logs, Investigations, SSH Keys, TOTP, Groups, Teams, Ambient, Settings
  const allModules: MenuItem[] = [
    {
      id: 'hosts',
      label: 'Hosts',
      href: '/',
      icon: 'M4 17l6-5-6-5M12 19h8',
      color: 'text-rose-500 bg-rose-500/10'
    },
    {
      id: 'session',
      label: 'Session',
      href: '/session',
      icon: 'M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z',
      color: 'text-emerald-500 bg-emerald-500/10'
    },
    {
      id: 'sftp',
      label: 'SFTP',
      href: '/sftp',
      icon: 'M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z',
      color: 'text-sky-500 bg-sky-500/10'
    },
    {
      id: 'snippets',
      label: 'Snippets',
      href: '/snippets',
      icon: 'M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4',
      color: 'text-amber-500 bg-amber-500/10'
    },
    {
      id: 'prompt-studio',
      label: 'Prompt Studio',
      href: '/prompt-studio',
      icon: 'M9.813 15.904L9 18.75l-.813-2.846a4.5 4.5 0 00-3.09-3.09L2.25 12l2.846-.813a4.5 4.5 0 003.09-3.09L9 5.25l.813 2.846a4.5 4.5 0 003.09 3.09L15.75 12l-2.846.813a4.5 4.5 0 00-3.09 3.09zM18.259 8.715L18 9.75l-.259-1.035a3.375 3.375 0 00-2.455-2.456L14.25 6l1.036-.259a3.375 3.375 0 002.455-2.456L18 2.25l.259 1.035a3.375 3.375 0 002.456 2.456L21.75 6l-1.035.259a3.375 3.375 0 00-2.456 2.456zM16.894 20.567L16.5 21.75l-.394-1.183a2.25 2.25 0 00-1.423-1.423L13.5 18.75l1.183-.394a2.25 2.25 0 001.423-1.423l.394-1.183.394 1.183a2.25 2.25 0 001.423 1.423l1.183.394-1.183.394a2.25 2.25 0 00-1.423 1.423z',
      color: 'text-purple-500 bg-purple-500/10'
    },
    {
      id: 'monitoring',
      label: 'Monitoring',
      href: '/monitoring',
      icon: 'M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z',
      color: 'text-indigo-500 bg-indigo-500/10'
    },
    {
      id: 'tasks',
      label: 'Tasks',
      href: '/tasks',
      icon: 'M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z',
      color: 'text-teal-500 bg-teal-500/10'
    },
    {
      id: 'port-fwd',
      label: 'Port Fwd',
      href: '/port-forwarding',
      icon: 'M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4',
      color: 'text-orange-500 bg-orange-500/10'
    },
    {
      id: 'logs',
      label: 'Logs',
      href: '/command-logs',
      icon: 'M4 6h16M4 12h16M4 18h16',
      color: 'text-neutral-500 bg-neutral-500/10'
    },
    {
      id: 'investigations',
      label: 'Investigations',
      href: '/investigations',
      icon: 'M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z',
      color: 'text-cyan-500 bg-cyan-500/10'
    },
    {
      id: 'ssh-keys',
      label: 'SSH Keys',
      href: '/ssh-keys',
      icon: 'M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z',
      color: 'text-yellow-500 bg-yellow-500/10'
    },
    {
      id: 'totp',
      label: 'TOTP',
      href: '/totp',
      icon: 'M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z',
      color: 'text-blue-500 bg-blue-500/10'
    },
    {
      id: 'groups',
      label: 'Groups',
      href: '/groups',
      icon: 'M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10',
      color: 'text-violet-500 bg-violet-500/10'
    },
    {
      id: 'security',
      label: 'Security',
      href: '/security',
      icon: 'M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z',
      color: 'text-emerald-500 bg-emerald-500/10'
    },
    {
      id: 'devops',
      label: 'DevOps Lab',
      href: '/devops',
      icon: 'M13 10V3L4 14h7v7l9-11h-7z',
      color: 'text-amber-500 bg-amber-500/10'
    },
    {
      id: 'teams',
      label: 'Teams',
      href: '/teams',
      icon: 'M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z',
      color: 'text-pink-500 bg-pink-500/10'
    },
    {
      id: 'ambient',
      label: 'Ambient',
      href: '/settings?tab=ambient',
      icon: 'M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z',
      color: 'text-fuchsia-500 bg-fuchsia-500/10'
    },
    {
      id: 'design-system',
      label: 'Design System Lab',
      href: '/design-system',
      icon: 'M7 21a4 4 0 01-4-4V5a2 2 0 012-2h4a2 2 0 012 2v12a4 4 0 01-4 4zm0 0h12a2 2 0 002-2v-4a2 2 0 00-2-2h-2.343M11 7.343l1.657-1.657a2 2 0 012.828 0l2.829 2.829a2 2 0 010 2.828l-8.486 8.485M7 17h.01',
      color: 'text-cyan-500 bg-cyan-500/10'
    },
    {
      id: 'settings',
      label: 'Settings',
      href: '/settings',
      icon: 'M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065zM15 12a3 3 0 11-6 0 3 3 0 016 0z',
      color: 'text-zinc-500 bg-zinc-500/10'
    }
  ];

  const filteredModules = $derived(
    searchQuery.trim() === ''
      ? allModules
      : allModules.filter((m) =>
          m.label.toLowerCase().includes(searchQuery.trim().toLowerCase())
        )
  );

  function navigateTo(href: string) {
    onClose();
    void goto(href);
  }
</script>

{#if open}
  <!-- Backdrop -->
  <div
    class="fixed inset-0 z-50 bg-black/60 backdrop-blur-sm transition-opacity md:hidden"
    onclick={onClose}
    onkeydown={(e) => e.key === 'Escape' && onClose()}
    tabindex="-1"
    role="button"
    aria-label="Tutup panel semua menu"
  ></div>

  <!-- Slide-up Bottom Sheet -->
  <div
    class="fixed inset-x-0 bottom-0 z-50 max-h-[85vh] bg-white dark:bg-[#141414] border-t border-neutral-200 dark:border-neutral-800 rounded-t-3xl shadow-2xl flex flex-col md:hidden pb-[max(1rem,env(safe-area-inset-bottom,0px))] transition-transform transform duration-200 ease-out"
    role="dialog"
    aria-modal="true"
    aria-label="Semua Menu Modul"
  >
    <!-- Drag handle indicator -->
    <div class="w-full flex justify-center pt-3 pb-2 cursor-grab active:cursor-grabbing">
      <div class="w-10 h-1 rounded-full bg-neutral-300 dark:bg-neutral-700"></div>
    </div>

    <!-- Header & Search filter bar -->
    <div class="px-5 pb-3 pt-1 space-y-3 shrink-0">
      <div class="flex items-center justify-between">
        <h2 class="text-base font-bold text-neutral-900 dark:text-white tracking-tight">Semua Menu Modul</h2>
        <button
          type="button"
          onclick={onClose}
          class="p-1.5 rounded-full hover:bg-neutral-100 dark:hover:bg-neutral-800 text-neutral-500 hover:text-neutral-900 dark:text-neutral-400 dark:hover:text-white transition-colors cursor-pointer"
          aria-label="Tutup"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Search Filter Bar -->
      <div class="relative">
        <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none text-neutral-400">
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
        </div>
        <input
          type="text"
          bind:value={searchQuery}
          placeholder="Cari modul atau menu..."
          class="w-full pl-9 pr-8 py-2 rounded-xl text-xs bg-neutral-100 dark:bg-neutral-800/70 border border-neutral-200 dark:border-neutral-700 text-neutral-900 dark:text-white placeholder-neutral-400 focus:outline-none focus:ring-2 focus:ring-sky-500/40"
        />
        {#if searchQuery}
          <button
            type="button"
            onclick={() => (searchQuery = '')}
            class="absolute inset-y-0 right-0 pr-3 flex items-center text-neutral-400 hover:text-neutral-600 dark:hover:text-neutral-200 cursor-pointer"
          >
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        {/if}
      </div>
    </div>

    <!-- 4x4 Modules Grid Area -->
    <div class="px-5 py-2 overflow-y-auto scrollbar-none flex-1 min-h-0">
      {#if filteredModules.length === 0}
        <div class="py-10 text-center text-xs text-neutral-500">
          Modul tidak ditemukan untuk "{searchQuery}"
        </div>
      {:else}
        <div class="grid grid-cols-4 gap-3 py-1">
          {#each filteredModules as item (item.id)}
            <button
              type="button"
              onclick={() => navigateTo(item.href)}
              class="flex flex-col items-center justify-center p-2.5 rounded-2xl hover:bg-neutral-100 dark:hover:bg-neutral-800/80 active:scale-95 transition-all text-center cursor-pointer group"
            >
              <div class="w-11 h-11 rounded-2xl flex items-center justify-center mb-1.5 shadow-sm {item.color} group-hover:scale-105 transition-transform">
                <svg class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d={item.icon} />
                </svg>
              </div>
              <span class="text-[11px] font-medium text-neutral-700 dark:text-neutral-300 group-hover:text-neutral-900 dark:group-hover:text-white line-clamp-1 break-all">
                {item.label}
              </span>
            </button>
          {/each}
        </div>
      {/if}
    </div>
  </div>
{/if}
