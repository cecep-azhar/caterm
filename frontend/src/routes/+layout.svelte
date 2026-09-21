<script lang="ts">
  import "../app.css";
  import LockScreen from '$lib/components/LockScreen.svelte';
  import NotificationCenter from '$lib/components/NotificationCenter.svelte';
  import Logo from '$lib/components/Logo.svelte';
  import WorkspaceMenu from '$lib/components/WorkspaceMenu.svelte';
  import { page } from '$app/state';
  import { getTabs, closeTab } from '$lib/stores/sessionTabs.svelte';
  import { getToasts, showToast } from '$lib/stores/uiNotifications.svelte';
  import { startMonitoring, stopMonitoring, monitorState } from '$lib/stores/monitorStore.svelte';
  import { onMount, onDestroy } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';

  let timeAgo = $state('never');
  
  $effect(() => {
    const updateTime = () => {
      if (!monitorState.lastUpdated) { timeAgo = 'never'; return; }
      const diff = Math.floor((Date.now() - monitorState.lastUpdated.getTime()) / 1000);
      if (diff < 2) timeAgo = 'just now';
      else if (diff < 60) timeAgo = `${diff}s ago`;
      else timeAgo = `${Math.floor(diff/60)}m ago`;
    };
    updateTime();
    const t = setInterval(updateTime, 1000);
    return () => clearInterval(t);
  });

  let isDarkTheme = $state(true);

  onMount(() => {
    startMonitoring();
    try {
      const savedTheme = localStorage.getItem('caterm-theme');
      if (savedTheme !== null) {
        isDarkTheme = savedTheme === 'dark';
      } else {
        isDarkTheme = true;
      }
      document.documentElement.classList.toggle('dark', isDarkTheme);
    } catch (e) {
      // ignore
    }
  });

  onDestroy(() => {
    stopMonitoring();
  });

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

  function toggleTheme() {
    isDarkTheme = !isDarkTheme;
    if (typeof document !== 'undefined') {
      document.documentElement.classList.toggle('dark', isDarkTheme);
    }
    try {
      localStorage.setItem('caterm-theme', isDarkTheme ? 'dark' : 'light');
    } catch (e) {
      // ignore
    }
  }

  let toastsList = $derived(getToasts());
  let unreadCount = $derived(toastsList.length);

  let { children } = $props();

  let isUnlocked = $state(false);
  let sessionTabs = $derived(getTabs());

  // Sidebar collapse & responsive mobile drawer state
  let isCollapsed = $state(false);
  let prevTabsCount = $state(0);
  let mobileDrawerOpen = $state(false);

  // Auto-collapse to icon-only mode when an active connection exists, expand when none
  $effect(() => {
    const currentCount = sessionTabs.length;
    if (prevTabsCount === 0 && currentCount > 0) {
      isCollapsed = true;
    } else if (prevTabsCount > 0 && currentCount === 0) {
      isCollapsed = false;
    }
    prevTabsCount = currentCount;
  });

  function toggleSidebar() {
    isCollapsed = !isCollapsed;
  }

  // Active route styling
  function isActive(href: string): boolean {
    if (href === '/') return page.url.pathname === '/';
    return page.url.pathname.startsWith(href);
  }

  // Maximize real estate on /session or active connections
  let isSessionActive = $derived(page.url.pathname.startsWith('/session') || sessionTabs.length > 0);

  const navItems = [
    {
      href: '/',
      label: 'Hosts',
      iconColor: 'text-sky-500 dark:text-sky-400',
      path: 'M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6'
    },
    {
      href: '/groups',
      label: 'Groups',
      iconColor: 'text-indigo-500 dark:text-indigo-400',
      path: 'M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z'
    },
    {
      href: '/snippets',
      label: 'Snippets',
      iconColor: 'text-emerald-500 dark:text-emerald-400',
      path: 'M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4'
    },
    {
      href: '/sftp',
      label: 'Files (SFTP)',
      iconColor: 'text-teal-500 dark:text-teal-400',
      path: 'M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z'
    },
    {
      href: '/teams',
      label: 'Teams',
      iconColor: 'text-blue-500 dark:text-blue-400',
      path: 'M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z'
    },
    {
      href: '/port-forwarding',
      label: 'Port forwarding',
      iconColor: 'text-amber-500 dark:text-amber-400',
      path: 'M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4'
    },
    {
      href: '/monitoring',
      label: 'Monitoring',
      iconColor: 'text-purple-500 dark:text-purple-400',
      path: 'M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z'
    },
    {
      href: '/command-logs',
      label: 'Command logs',
      iconColor: 'text-gray-500 dark:text-gray-400',
      path: 'M4 6h16M4 12h16M4 18h16'
    },
    {
      href: '/investigations',
      label: 'Investigations',
      iconColor: 'text-orange-500 dark:text-orange-400',
      path: 'M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z'
    },
    {
      href: '/ssh-keys',
      label: 'SSH keys',
      iconColor: 'text-rose-500 dark:text-rose-400',
      path: 'M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z'
    }
  ];
</script>

<svelte:window onkeydown={(e) => { if (e.key === 'Escape') mobileDrawerOpen = false; }} />

<NotificationCenter />

{#if !isUnlocked}
  <LockScreen onUnlocked={() => isUnlocked = true} />
{:else}
<div class="flex h-screen bg-neutral-100 dark:bg-[#0e0e0e] text-neutral-800 dark:text-neutral-300 font-sans transition-colors duration-150">
  <!-- Mobile Slide-out Drawer Backdrop -->
  {#if mobileDrawerOpen}
    <button
      type="button"
      onclick={() => mobileDrawerOpen = false}
      class="fixed inset-0 z-40 bg-black/60 backdrop-blur-sm md:hidden transition-opacity border-0 p-0 cursor-default"
      aria-label="Close menu backdrop"
    ></button>
  {/if}

  <!-- Mobile Slide-out Drawer Navigation -->
  <aside
    class="fixed inset-y-0 left-0 z-50 w-72 max-w-[85vw] bg-white dark:bg-[#0e0e0e] border-r border-neutral-200 dark:border-neutral-800 flex flex-col justify-between shadow-2xl md:hidden transform transition-transform duration-200 ease-in-out {mobileDrawerOpen ? 'translate-x-0' : '-translate-x-full'}"
    aria-label="Mobile Navigation"
  >
    <div>
      <div class="h-11 border-b border-neutral-200 dark:border-neutral-800 flex items-center justify-between px-4">
        <div class="flex items-center gap-2">
          <Logo size={22} mode="brand" />
          <span class="font-bold text-neutral-900 dark:text-white text-base tracking-wide">CATerm</span>
          <span class="text-[10px] px-1.5 py-0.5 rounded bg-sky-500/10 dark:bg-sky-500/20 text-sky-600 dark:text-sky-400 font-mono">v2.0.9</span>
        </div>
        <button
          onclick={() => mobileDrawerOpen = false}
          class="p-1.5 rounded-lg hover:bg-neutral-100 dark:hover:bg-neutral-800 text-neutral-500 hover:text-neutral-900 dark:text-neutral-400 dark:hover:text-white transition-colors"
          aria-label="Close navigation"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <nav class="p-3 space-y-1 overflow-y-auto max-h-[calc(100vh-160px)] text-sm font-medium">
        {#each navItems as item}
          <a
            href={item.href}
            onclick={() => mobileDrawerOpen = false}
            title={item.label}
            class="p-2.5 rounded-lg flex items-center gap-3 transition-colors {isActive(item.href) ? 'bg-neutral-200/70 dark:bg-neutral-800 text-neutral-900 dark:text-white font-semibold' : 'text-neutral-600 dark:text-neutral-400 hover:bg-neutral-100 dark:hover:bg-neutral-800/80 hover:text-neutral-900 dark:hover:text-white'}"
          >
            <svg class="w-5 h-5 shrink-0 {item.iconColor}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={item.path} />
            </svg>
            <span class="truncate">{item.label}</span>
          </a>
        {/each}
      </nav>
    </div>

    <!-- Mobile Drawer Footer -->
    <div class="p-3 space-y-2 border-t border-neutral-200 dark:border-neutral-800/80">
      <a
        href="/settings"
        onclick={() => mobileDrawerOpen = false}
        title="Settings"
        class="p-2.5 rounded-lg flex items-center gap-3 transition-colors {isActive('/settings') ? 'bg-neutral-200/70 dark:bg-neutral-800 text-neutral-900 dark:text-white font-semibold' : 'text-neutral-600 dark:text-neutral-400 hover:bg-neutral-100 dark:hover:bg-neutral-800/80 hover:text-neutral-900 dark:hover:text-white'}"
      >
        <svg class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"></path>
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"></path>
        </svg>
        <span class="truncate text-sm font-medium">Settings</span>
      </a>

      <div class="p-2 rounded-lg bg-neutral-100 dark:bg-neutral-900/60 border border-neutral-200 dark:border-neutral-800/50 flex items-center justify-between">
        <div class="flex items-center gap-2 overflow-hidden">
          <div class="w-8 h-8 rounded-full shrink-0 bg-sky-500/15 border border-sky-500/30 flex items-center justify-center text-sky-600 dark:text-sky-400">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
            </svg>
          </div>
          <div class="truncate text-xs flex flex-col justify-center">
            <div class="flex items-center gap-1.5">
              <span class="font-medium text-neutral-900 dark:text-white truncate">Vault Admin</span>
              <span class="text-[9px] font-bold px-1 py-0.5 bg-emerald-600 text-white rounded">LOCAL</span>
            </div>
            <p class="text-[10px] text-neutral-500 dark:text-neutral-400 truncate">Encrypted Local Vault</p>
          </div>
        </div>
        <button
          onclick={() => { mobileDrawerOpen = false; isUnlocked = false; }}
          title="Lock Vault"
          aria-label="Lock Vault"
          class="p-1.5 text-neutral-500 hover:text-amber-600 dark:text-neutral-400 dark:hover:text-amber-400 hover:bg-neutral-200 dark:hover:bg-neutral-800 rounded transition-colors shrink-0"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"/></svg>
        </button>
      </div>
    </div>
  </aside>

  <!-- Desktop Sidebar -->
  <aside
    class="hidden md:flex flex-col justify-between shrink-0 bg-white dark:bg-[#0e0e0e] border-r border-neutral-200 dark:border-neutral-800 transition-all duration-200 ease-in-out {isCollapsed ? 'w-16' : 'w-64'}"
    aria-label="Sidebar"
  >
    <div>
      <!-- Sidebar Header -->
      <div class="h-11 border-b border-neutral-200 dark:border-neutral-800 flex items-center {isCollapsed ? 'justify-center px-2' : 'justify-between px-3'} transition-all">
        {#if isCollapsed}
          <button
            type="button"
            onclick={toggleSidebar}
            title="Expand sidebar"
            aria-label="Expand sidebar"
            class="p-1 rounded-lg hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors"
          >
            <Logo size={22} mode="brand" />
          </button>
        {:else}
          <div class="flex items-center gap-2 overflow-hidden min-w-0">
            <Logo size={22} mode="brand" />
            <span class="font-bold text-neutral-900 dark:text-white text-base tracking-wide truncate">CATerm</span>
            <span class="text-[10px] px-1.5 py-0.5 rounded bg-sky-500/10 dark:bg-sky-500/20 text-sky-600 dark:text-sky-400 font-mono shrink-0">v2.0.9</span>
          </div>
          <button
            type="button"
            onclick={toggleSidebar}
            title="Collapse sidebar"
            aria-label="Collapse sidebar"
            class="p-1 rounded hover:bg-neutral-100 dark:hover:bg-neutral-800 text-neutral-400 hover:text-neutral-700 dark:hover:text-neutral-200 transition-colors"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 19l-7-7 7-7m8 14l-7-7 7-7" />
            </svg>
          </button>
        {/if}
      </div>
      
      <!-- Nav Links -->
      <nav class="p-2 space-y-1 text-sm font-medium">
        {#each navItems as item}
          <a
            href={item.href}
            title={item.label}
            class="p-2.5 rounded-lg flex items-center {isCollapsed ? 'justify-center' : 'gap-3'} transition-colors {isActive(item.href) ? 'bg-neutral-200/70 dark:bg-neutral-800 text-neutral-900 dark:text-white font-semibold' : 'text-neutral-600 dark:text-neutral-400 hover:bg-neutral-100 dark:hover:bg-neutral-800/80 hover:text-neutral-900 dark:hover:text-white'}"
          >
            <svg class="w-5 h-5 shrink-0 {item.iconColor}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={item.path} />
            </svg>
            {#if !isCollapsed}
              <span class="truncate">{item.label}</span>
            {/if}
          </a>
        {/each}
      </nav>
    </div>

    <!-- Desktop Bottom Actions: Settings, Lock & Manual Collapse Toggle -->
    <div class="p-2 space-y-1 border-t border-neutral-200 dark:border-neutral-800/80">
      <a
        href="/settings"
        title="Settings"
        class="p-2.5 rounded-lg flex items-center {isCollapsed ? 'justify-center' : 'gap-3'} transition-colors {isActive('/settings') ? 'bg-neutral-200/70 dark:bg-neutral-800 text-neutral-900 dark:text-white font-semibold' : 'text-neutral-600 dark:text-neutral-400 hover:bg-neutral-100 dark:hover:bg-neutral-800/80 hover:text-neutral-900 dark:hover:text-white'}"
      >
        <svg class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"></path>
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"></path>
        </svg>
        {#if !isCollapsed}
          <span class="truncate">Settings</span>
        {/if}
      </a>

      <!-- Profile & Lock Button -->
      <div class="p-1.5 rounded-lg bg-neutral-100 dark:bg-neutral-900/60 border border-neutral-200 dark:border-neutral-800/50 flex items-center {isCollapsed ? 'justify-center' : 'justify-between'}">
        {#if !isCollapsed}
          <div class="flex items-center gap-2 overflow-hidden">
            <div class="w-7 h-7 rounded-full shrink-0 bg-sky-500/15 border border-sky-500/30 flex items-center justify-center text-sky-600 dark:text-sky-400">
              <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
              </svg>
            </div>
            <div class="truncate text-xs flex flex-col justify-center">
              <div class="flex items-center gap-1.5">
                <span class="font-medium text-neutral-900 dark:text-white truncate">Vault Admin</span>
                <span class="text-[8px] font-bold px-1 py-0.5 bg-emerald-600 text-white rounded leading-none">LOCAL</span>
              </div>
              <p class="text-[10px] text-neutral-500 dark:text-neutral-400 truncate">Encrypted Local Vault</p>
            </div>
          </div>
        {/if}
        <button
          onclick={() => isUnlocked = false}
          title="Lock Vault"
          aria-label="Lock Vault"
          class="p-1.5 text-neutral-500 hover:text-amber-600 dark:text-neutral-400 dark:hover:text-amber-400 hover:bg-neutral-200 dark:hover:bg-neutral-800 rounded transition-colors shrink-0"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"/></svg>
        </button>
      </div>

      <!-- Manual Collapse Toggle Button -->
      <button
        type="button"
        onclick={toggleSidebar}
        title={isCollapsed ? "Expand sidebar" : "Collapse sidebar"}
        aria-label={isCollapsed ? "Expand sidebar" : "Collapse sidebar"}
        class="w-full p-2 rounded-lg hover:bg-neutral-100 dark:hover:bg-neutral-800/80 flex items-center {isCollapsed ? 'justify-center' : 'gap-3'} transition-colors text-neutral-500 hover:text-neutral-900 dark:text-neutral-400 dark:hover:text-white"
      >
        {#if isCollapsed}
          <svg class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 5l7 7-7 7M5 5l7 7-7 7" />
          </svg>
        {:else}
          <svg class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 19l-7-7 7-7m8 14l-7-7 7-7" />
          </svg>
          <span class="text-xs font-medium truncate">Collapse sidebar</span>
        {/if}
      </button>
    </div>
  </aside>

  <!-- Main Content -->
  <main class="flex-1 flex flex-col h-screen overflow-hidden bg-neutral-100 dark:bg-[#0a0a0a] transition-colors duration-150">
    <!-- Top Bar (Compact Header) -->
    <header class="h-11 border-b border-neutral-200 dark:border-neutral-800 flex items-center px-3 md:px-4 bg-white dark:bg-neutral-900 shrink-0 select-none transition-colors duration-150 justify-between gap-2" data-tauri-drag-region>
      <div class="flex items-center gap-1 md:gap-2 min-w-0">
        <!-- Mobile Menu Hamburger Button -->
        <button
          type="button"
          onclick={() => mobileDrawerOpen = true}
          class="md:hidden p-1.5 rounded-lg text-neutral-600 dark:text-neutral-300 hover:text-neutral-900 dark:hover:text-white hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors shrink-0"
          title="Open navigation menu"
          aria-label="Open navigation menu"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
          </svg>
        </button>

        <!-- Sessions Tab System -->
        <div class="flex gap-1.5 md:gap-2 text-xs md:text-sm items-center pointer-events-auto overflow-x-auto min-w-0 py-0.5">
          <a
            href="/"
            class="px-2.5 md:px-3 py-1 bg-neutral-100 dark:bg-neutral-800 text-neutral-900 dark:text-white rounded-t-md border-t border-l border-r border-neutral-300 dark:border-neutral-700 font-medium shrink-0"
          >
            Dashboard
          </a>
          {#each sessionTabs as tab (tab.id)}
            <div class="flex items-center gap-1 px-1 text-neutral-600 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200 hover:bg-neutral-100 dark:hover:bg-neutral-800/50 rounded-t-md transition-colors shrink-0">
              <a href="/session" class="py-1 pl-2 truncate max-w-[120px] md:max-w-[180px]">{tab.host.label}</a>
              <button
                onclick={() => closeTab(tab.id)}
                class="p-0.5 rounded hover:bg-neutral-200 dark:hover:bg-neutral-700 hover:text-rose-600 dark:hover:text-rose-400"
                title="Tutup sesi {tab.host.label}">
                <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-width="2" stroke-linecap="round" d="M6 18L18 6M6 6l12 12"></path></svg>
              </button>
            </div>
          {/each}
        </div>
      </div>

      <div class="flex items-center ml-auto gap-2 md:gap-3 text-neutral-500 dark:text-neutral-400 shrink-0">
        <!-- Workspaces Menu -->
        <WorkspaceMenu />

        <div class="w-px h-4 bg-neutral-200 dark:bg-neutral-700 mx-0.5 md:mx-1"></div>

        <!-- Live Status Indicator -->
        <div class="flex items-center gap-1.5 text-xs">
          <span class="text-green-500 animate-pulse text-[10px]" class:opacity-50={monitorState.isPolling}>●</span>
          <span class="font-mono text-neutral-700 dark:text-neutral-300 hidden sm:inline">{timeAgo}</span>
        </div>
        
        <div class="w-px h-4 bg-neutral-200 dark:bg-neutral-700 mx-0.5 md:mx-1"></div>

        <!-- Theme Toggle -->
        <button onclick={toggleTheme} class="p-1.5 hover:text-neutral-900 dark:hover:text-white hover:bg-neutral-100 dark:hover:bg-neutral-800 rounded transition-colors" title="Toggle Theme ({isDarkTheme ? 'Dark' : 'Light'})">
          {#if isDarkTheme}
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z"></path></svg>
          {:else}
            <svg class="w-4 h-4 text-amber-500" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z"></path></svg>
          {/if}
        </button>

        <!-- Notification Bell -->
        <button onclick={() => showToast('No new system notifications.', 'info')} class="p-1.5 hover:text-neutral-900 dark:hover:text-white hover:bg-neutral-100 dark:hover:bg-neutral-800 rounded transition-colors relative" title="Notifications">
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9"></path></svg>
          {#if unreadCount > 0}
            <span class="absolute top-0 right-0 w-2 h-2 bg-rose-500 rounded-full border border-white dark:border-neutral-900 animate-pulse"></span>
          {/if}
        </button>

        <div class="w-px h-4 bg-neutral-200 dark:bg-neutral-700 mx-0.5 md:mx-1 hidden sm:block"></div>

        <!-- Custom Window Controls -->
        <div class="hidden sm:flex items-center">
          <button onclick={() => minimizeWindow()} class="p-2 hover:bg-neutral-100 dark:hover:bg-neutral-800 text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white transition-colors" title="Minimize">
            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 12H4"></path></svg>
          </button>
          <button onclick={() => maximizeWindow()} class="p-2 hover:bg-neutral-100 dark:hover:bg-neutral-800 text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white transition-colors" title="Maximize">
            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 8h16M4 16h16M8 4v16M16 4v16" stroke-dasharray="2 2"></path></svg>
          </button>
          <button onclick={() => closeWindow()} class="p-2 hover:bg-rose-500 hover:text-white text-neutral-500 dark:text-neutral-400 transition-colors rounded-tr" title="Close">
            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path></svg>
          </button>
        </div>
      </div>
    </header>
    
    <!-- Content Area (Screen Real Estate Optimized) -->
    <div class="flex-1 overflow-auto bg-neutral-100 dark:bg-[#0a0a0a] text-neutral-900 dark:text-neutral-100 relative transition-colors duration-150 {isSessionActive ? 'p-0 md:p-1' : 'p-3 md:p-6'}">
      {@render children()}
    </div>
  </main>
</div>
{/if}
