<script lang="ts">
  import "../app.css";
  import LockScreen from '$lib/components/LockScreen.svelte';
  import NotificationCenter from '$lib/components/NotificationCenter.svelte';
  import Logo from '$lib/components/Logo.svelte';
  
  import { getTabs, closeTab } from '$lib/stores/sessionTabs.svelte';
  import { getToasts, showToast } from '$lib/stores/uiNotifications.svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';

  const appWindow = typeof window !== 'undefined' && (window as any).__TAURI__ ? getCurrentWindow() : null;

  async function minimizeWindow() {
    if (appWindow) await appWindow.minimize();
  }
  async function maximizeWindow() {
    if (appWindow) await appWindow.toggleMaximize();
  }
  async function closeWindow() {
    if (appWindow) await appWindow.close();
  }

  let isDarkTheme = $state(true);
  function toggleTheme() {
    isDarkTheme = !isDarkTheme;
    if (typeof document !== 'undefined') {
      document.documentElement.classList.toggle('dark', isDarkTheme);
    }
  }

  let toastsList = $derived(getToasts());
  let unreadCount = $derived(toastsList.length);


  let { children } = $props();

  let isUnlocked = $state(false);
  let sessionTabs = $derived(getTabs());
</script>

<NotificationCenter />

{#if !isUnlocked}
  <LockScreen onUnlocked={() => isUnlocked = true} />
{:else}
<div class="flex h-screen bg-[#0e0e0e] text-neutral-300 font-sans">
  <!-- Sidebar -->
  <aside class="w-16 md:w-64 border-r border-neutral-800 flex flex-col justify-between shrink-0">
    <div>
      <div class="h-12 border-b border-neutral-800 flex items-center justify-between px-4">
        <div class="flex items-center gap-2">
          <Logo size={24} mode="dark" />
          <span class="font-bold text-white text-lg tracking-wide hidden md:block">CATerm</span>
        </div>
        <span class="text-[10px] px-1.5 py-0.5 rounded bg-sky-500/20 text-sky-400 font-mono hidden md:block">v2.0.4</span>
      </div>
      
      
      <nav class="p-2 space-y-1 text-sm font-medium">
        <a href="/" class="p-2.5 rounded-lg hover:bg-neutral-800/80 flex items-center gap-3 transition-colors text-neutral-200 hover:text-white">
          <svg class="w-5 h-5 shrink-0 text-sky-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6"></path></svg>
          <span class="hidden md:inline">Hosts</span>
        </a>
        <a href="/groups" class="p-2.5 rounded-lg hover:bg-neutral-800/80 flex items-center gap-3 transition-colors text-neutral-400 hover:text-white">
          <svg class="w-5 h-5 shrink-0 text-indigo-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"></path></svg>
          <span class="hidden md:inline">Groups</span>
        </a>
        <a href="/snippets" class="p-2.5 rounded-lg hover:bg-neutral-800/80 flex items-center gap-3 transition-colors text-neutral-400 hover:text-white">
          <svg class="w-5 h-5 shrink-0 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4"></path></svg>
          <span class="hidden md:inline">Snippets</span>
        </a>
        <a href="/teams" class="p-2.5 rounded-lg hover:bg-neutral-800/80 flex items-center gap-3 transition-colors text-neutral-400 hover:text-white">
          <svg class="w-5 h-5 shrink-0 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"></path></svg>
          <span class="hidden md:inline">Teams</span>
        </a>
        <a href="/port-forwarding" class="p-2.5 rounded-lg hover:bg-neutral-800/80 flex items-center gap-3 transition-colors text-neutral-400 hover:text-white">
          <svg class="w-5 h-5 shrink-0 text-amber-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4"></path></svg>
          <span class="hidden md:inline">Port forwarding</span>
        </a>
        <a href="/monitoring" class="p-2.5 rounded-lg hover:bg-neutral-800/80 flex items-center gap-3 transition-colors text-neutral-400 hover:text-white">
          <svg class="w-5 h-5 shrink-0 text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"></path></svg>
          <span class="hidden md:inline">Monitoring</span>
        </a>
        <a href="/command-logs" class="p-2.5 rounded-lg hover:bg-neutral-800/80 flex items-center gap-3 transition-colors text-neutral-400 hover:text-white">
          <svg class="w-5 h-5 shrink-0 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"></path></svg>
          <span class="hidden md:inline">Command logs</span>
        </a>
        <a href="/investigations" class="p-2.5 rounded-lg hover:bg-neutral-800/80 flex items-center gap-3 transition-colors text-neutral-400 hover:text-white">
          <svg class="w-5 h-5 shrink-0 text-orange-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"></path></svg>
          <span class="hidden md:inline">Investigations</span>
        </a>
        <a href="/ssh-keys" class="p-2.5 rounded-lg hover:bg-neutral-800/80 flex items-center gap-3 transition-colors text-neutral-400 hover:text-white">
          <svg class="w-5 h-5 shrink-0 text-rose-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z"></path></svg>
          <span class="hidden md:inline">SSH keys</span>
        </a>
      </nav>
    </div>

    <!-- Bottom Actions: Settings & User Identity / Lock -->
    <div class="p-2 space-y-1 border-t border-neutral-800/80">
      <a href="/settings" class="p-2.5 rounded-lg hover:bg-neutral-800/80 flex items-center gap-3 transition-colors text-neutral-400 hover:text-white">
        <svg class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"></path><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"></path></svg>
        <span class="hidden md:inline">Settings</span>
      </a>

      <!-- Profile & Lock Button -->
      <div class="p-2 rounded-lg bg-neutral-900/60 border border-neutral-800/50 flex items-center justify-between">
        <div class="hidden md:flex items-center gap-2 overflow-hidden">
          <img src="https://ui-avatars.com/api/?name=User&background=0D8ABC&color=fff" alt="Avatar" class="w-8 h-8 rounded-full shrink-0 border border-neutral-700" />
          <div class="truncate text-xs flex flex-col justify-center">
            <div class="flex items-center gap-1.5">
              <span class="font-medium text-white truncate">John Doe</span>
              <span class="text-[9px] font-bold px-1 py-0.5 bg-blue-500 text-white rounded">FREE</span>
            </div>
            <p class="text-[10px] text-neutral-400 truncate">john@example.com</p>
          </div>
        </div>
        <button onclick={() => isUnlocked = false} title="Lock Vault" class="p-1.5 text-neutral-400 hover:text-amber-400 hover:bg-neutral-800 rounded transition-colors shrink-0">
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"/></svg>
        </button>
      </div>
    </div>
  </aside>

  <!-- Main Content -->
  <main class="flex-1 flex flex-col h-screen overflow-hidden">
    <!-- Top Bar (Sessions Tab System Placeholder) -->
    <header class="h-12 border-b border-neutral-800 flex items-center px-4 bg-neutral-900 shrink-0 select-none" data-tauri-drag-region>
      <div class="flex gap-2 text-sm items-center pointer-events-auto">
        <a href="/" class="px-3 py-1 bg-neutral-800 text-white rounded-t-md border-t border-l border-r border-neutral-700 font-medium">Dashboard</a>
        {#each sessionTabs as tab (tab.id)}
          <div class="flex items-center gap-1 px-1 text-neutral-400 hover:text-neutral-200 hover:bg-neutral-800/50 rounded-t-md transition-colors">
            <a href="/session" class="py-1 pl-2">{tab.host.label}</a>
            <button
              onclick={() => closeTab(tab.id)}
              class="p-0.5 rounded hover:bg-neutral-700 hover:text-rose-400"
              title="Tutup sesi {tab.host.label}">
              <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-width="2" stroke-linecap="round" d="M6 18L18 6M6 6l12 12"></path></svg>
            </button>
          </div>
        {/each}
      </div>

      <div class="flex items-center ml-auto gap-3 text-neutral-400 shrink-0">
        <!-- Live Status Indicator -->
        <div class="flex items-center gap-1.5 text-xs">
          <span class="text-green-500 animate-pulse text-[10px]">●</span>
          <span class="font-mono">just now</span>
        </div>
        
        <div class="w-px h-4 bg-neutral-700 mx-1"></div>

        <!-- Theme Toggle -->
        <button onclick={toggleTheme} class="p-1 hover:text-white hover:bg-neutral-800 rounded transition-colors" title="Toggle Theme">
          {#if isDarkTheme}
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z"></path></svg>
          {:else}
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z"></path></svg>
          {/if}
        </button>

        <!-- Notification Bell -->
        <button onclick={() => showToast('No new system notifications.', 'info')} class="p-1 hover:text-white hover:bg-neutral-800 rounded transition-colors relative" title="Notifications">
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9"></path></svg>
          {#if unreadCount > 0}
            <span class="absolute top-0 right-0 w-2 h-2 bg-rose-500 rounded-full border border-neutral-900 animate-pulse"></span>
          {/if}
        </button>

        <div class="w-px h-4 bg-neutral-700 mx-1"></div>

        <!-- Custom Window Controls -->
        <div class="flex items-center">
          <button onclick={() => minimizeWindow()} class="p-2 hover:bg-neutral-800 transition-colors" title="Minimize">
            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 12H4"></path></svg>
          </button>
          <button onclick={() => maximizeWindow()} class="p-2 hover:bg-neutral-800 transition-colors" title="Maximize">
            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 8h16M4 16h16M8 4v16M16 4v16" stroke-dasharray="2 2"></path></svg>
          </button>
          <button onclick={() => closeWindow()} class="p-2 hover:bg-rose-500 hover:text-white transition-colors rounded-tr" title="Close">
            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path></svg>
          </button>
        </div>
      </div>
    </header>
    
    <div class="flex-1 overflow-auto bg-[#0a0a0a] p-6 relative">
      {@render children()}
    </div>
  </main>
</div>
{/if}
