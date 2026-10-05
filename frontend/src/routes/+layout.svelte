<script lang="ts">
  import "../app.css";
  import LockScreen from '$lib/components/LockScreen.svelte';
  import { onVaultUnlocked, onVaultLocked } from '$lib/stores/pro.svelte';
  import HeaderQuickControls from '$lib/components/HeaderQuickControls.svelte';
  import NotificationCenter from '$lib/components/NotificationCenter.svelte';
  import Logo from '$lib/components/Logo.svelte';
  import WorkspaceMenu from '$lib/components/WorkspaceMenu.svelte';
  import AiChatPanel from '$lib/components/AiChatPanel.svelte';
  import SessionViewport from '$lib/components/SessionViewport.svelte';
  import { getAiChatState, toggleAiChat, closeAiChat } from '$lib/stores/aiChat.svelte';
  import { page } from '$app/state';
  import { getTabs, closeSessionTab, closeAllTabs, renameTab, tabLabel, setTabGroup, closeTabsInGroup, GROUP_COLORS } from '$lib/stores/sessionTabs.svelte';
  import {
    getSessionView,
    setLayout,
    setSelectedTabId,
    setShowFiles,
    type PaneLayout
  } from '$lib/stores/sessionView.svelte';
  import {
  	getTheme,
  	initTheme,
  	setTheme
  } from '$lib/stores/theme.svelte';
  import { getToasts, showToast, confirmModal } from '$lib/stores/uiNotifications.svelte';
  import { startMonitoring, stopMonitoring, monitorState } from '$lib/stores/monitorStore.svelte';
  import FeedbackModal from '$lib/components/FeedbackModal.svelte';
  import CrashReportModal from '$lib/components/CrashReportModal.svelte';
  import { getPendingCrashReport, type ScrubbedCrashReport } from '$lib/api/crash';
  import ProfileMenu from '$lib/components/ProfileMenu.svelte';
  import { getFeedbackPromptState } from '$lib/stores/feedbackStore.svelte';
  import { checkForUpdates } from '$lib/stores/updater.svelte';
  import { lockVault } from '$lib/api/vault';
  import { APP_VERSION } from '$lib/appInfo';
  import { goto } from '$app/navigation';
  import { onMount, onDestroy } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { invoke } from '@tauri-apps/api/core';
  import { openExternalUrl } from '$lib/utils/url';
  import { t } from '$lib/i18n/index.svelte';
  import LanguageSwitcher from '$lib/components/LanguageSwitcher.svelte';
  import { navItems as getNavItems, settingsNavItem } from '$lib/navItems';
  import CommandPalette from '$lib/components/CommandPalette.svelte';
  import { getPalette, openPalette, closePalette } from '$lib/stores/commandPalette.svelte';
  import { appCommandFor, type AppCommand } from '$lib/shortcuts';

  // Seconds since the last monitor poll; the label is derived so it re-renders the moment the
  // language changes instead of waiting for the next 5 s tick.
  let syncAgeSeconds = $state<number | null>(null);

  $effect(() => {
    const updateAge = () => {
      if (typeof document !== 'undefined' && document.hidden) return;
      syncAgeSeconds = monitorState.lastUpdated
        ? Math.floor((Date.now() - monitorState.lastUpdated.getTime()) / 1000)
        : null;
    };
    updateAge();
    const timer = setInterval(updateAge, 5000);
    return () => clearInterval(timer);
  });

  const timeAgo = $derived(
    syncAgeSeconds === null
      ? t('time.never')
      : syncAgeSeconds < 2
        ? t('time.justNow')
        : syncAgeSeconds < 60
          ? t('time.secondsAgo', { n: syncAgeSeconds })
          : t('time.minutesAgo', { n: Math.floor(syncAgeSeconds / 60) })
  );

  const theme = getTheme();
  const aiChat = getAiChatState();
  const feedbackPrompt = getFeedbackPromptState();
  const isDarkTheme = $derived(theme.name === 'dark');

  // Reviewed once per launch, after the previous run's panic hook (if any fired) had a chance
  // to finish writing its dump. Null unless there is something pending — see
  // caterm-crash-reporting-spec-v1.md §7. Never surfaced again this session once dismissed.
  let pendingCrashReport = $state<ScrubbedCrashReport | null>(null);

  onMount(() => {
    startMonitoring();
    initTheme();
    // One quiet check per launch; failures (offline, no release yet) stay silent. Skipped under
    // `vite dev`, where there is no installed build to update.
    if (!import.meta.env.DEV) void checkForUpdates({ silent: true });

    void getPendingCrashReport()
      .then((report) => {
        pendingCrashReport = report;
      })
      .catch(() => {
        // No dump, disabled, or an unreadable data dir — silently nothing to show either way.
      });

    // Phone in landscape (including rotating into it): height is the scarce axis, so switch
    // to the icon-only sidebar. Leaving landscape keeps whatever the user picked.
    const shortViewport = window.matchMedia('(max-height: 500px)');
    const collapseWhenShort = () => {
      if (shortViewport.matches) isCollapsed = true;
    };
    collapseWhenShort();
    shortViewport.addEventListener('change', collapseWhenShort);

    const handleGlobalClick = (e: MouseEvent) => {
      const target = (e.target as HTMLElement)?.closest('a');
      if (target && target.href) {
        const href = target.href;
        const isInternal = href.startsWith(window.location.origin) || href.includes('tauri.localhost') || href.includes('ipc.localhost') || href.startsWith('/') || href.startsWith('#');
        if (!isInternal && (href.startsWith('http://') || href.startsWith('https://'))) {
          e.preventDefault();
          e.stopPropagation();
          openExternalUrl(href);
        }
      }
    };
    window.addEventListener('click', handleGlobalClick, true);

    // No stray WebView context menu ("Save link as", "Inspect"...). Right-click only does
    // something where CATerm defines its own menu (session tabs, SFTP file lists) — those
    // handlers still run, this only suppresses the native one. Text fields keep the native
    // cut/copy/paste menu; the terminal's hidden xterm textarea does not.
    const handleContextMenu = (e: MouseEvent) => {
      const target = e.target as HTMLElement | null;
      const editable = target?.closest('input, textarea, [contenteditable="true"]');
      if (editable && !editable.closest('.xterm')) return;
      e.preventDefault();
    };
    document.addEventListener('contextmenu', handleContextMenu, true);

    // Capture phase: runs before xterm (which would otherwise send the keys to the shell) and
    // before any page handler. See $lib/shortcuts for which keys are app-level.
    window.addEventListener('keydown', handleShortcut, true);

    return () => {
      window.removeEventListener('keydown', handleShortcut, true);
      window.removeEventListener('click', handleGlobalClick, true);
      document.removeEventListener('contextmenu', handleContextMenu, true);
      shortViewport.removeEventListener('change', collapseWhenShort);
    };
  });

  onDestroy(() => {
    stopMonitoring();
  });

  const isTauri = typeof window !== 'undefined' && Boolean((window as any).__TAURI_INTERNALS__ || (window as any).__TAURI__);
  const appWindow = isTauri ? getCurrentWindow() : null;

  async function minimizeWindow() {
    try {
      await invoke('window_minimize');
    } catch (e) {
      try {
        if (appWindow) {
          await appWindow.minimize();
        } else if (typeof window !== 'undefined') {
          const win = getCurrentWindow();
          await win.minimize();
        }
      } catch (err) {
        console.warn('Failed to minimize window:', err);
      }
    }
  }
  async function maximizeWindow() {
    try {
      await invoke('window_maximize');
    } catch (e) {
      try {
        if (appWindow) {
          await appWindow.toggleMaximize();
        } else if (typeof window !== 'undefined') {
          const win = getCurrentWindow();
          await win.toggleMaximize();
        }
      } catch (err) {
        console.warn('Failed to toggle maximize window:', err);
      }
    }
  }
  async function closeWindow() {
  	const activeCount = sessionTabs.length;
  	const message =
  		activeCount > 0
  			? t('shell.closeConfirmActiveSessions', { count: activeCount })
  			: t('shell.closeConfirm');
  	const confirmed = await confirmModal(
  		message,
  		t('shell.closeWindow'),
  		activeCount > 0,
  		t('shell.close'),
  		t('common.cancel')
  	);
  	if (!confirmed) return;

  	try {
  		await invoke('window_close');
  	} catch (e) {
  		try {
  			if (appWindow) {
  				await appWindow.close();
  			} else if (typeof window !== 'undefined') {
  				const win = getCurrentWindow();
  				await win.close();
  			}
  		} catch (err) {
  			console.warn('Failed to close window:', err);
  		}
  	}
  }

  async function startDragging(e: MouseEvent) {
    if (e.button !== 0) return;
    const target = e.target as HTMLElement | null;
    if (target?.closest('button, input, textarea, a, select, [role="button"], .no-drag')) return;
    try {
      await invoke('window_start_dragging');
    } catch {
      try {
        if (appWindow) {
          await appWindow.startDragging();
        } else if (typeof window !== 'undefined') {
          const win = getCurrentWindow();
          await win.startDragging();
        }
      } catch (err) {
        console.warn('Failed to start dragging window:', err);
      }
    }
  }

  let toastsList = $derived(getToasts());
  let unreadCount = $derived(toastsList.length);

  let { children } = $props();

  let isUnlocked = $state(false);
  let sessionTabs = $derived(getTabs());

  // Terminal workspace controls render in this one header rather than a second bar owned by
  // the /session route, so the layout reads the same shared state the route does.
  const view = getSessionView();

  const splitOptions: { value: PaneLayout; minTabs: number; title: string; path: string }[] = $derived([
    { value: 1, minTabs: 1, title: t('shell.splitSingle'), path: '' },
    {
      value: 2,
      minTabs: 2,
      title: t('shell.splitHorizontal'),
      path: 'M3 12h18M5 3h14a2 2 0 012 2v14a2 2 0 01-2 2H5a2 2 0 01-2-2V5a2 2 0 012-2z'
    },
    {
      value: 3,
      minTabs: 2,
      title: t('shell.splitVertical'),
      path: 'M12 3v18M5 3h14a2 2 0 012 2v14a2 2 0 01-2 2H5a2 2 0 01-2-2V5a2 2 0 012-2z'
    },
    {
      value: 4,
      minTabs: 3,
      title: t('shell.splitGrid'),
      path: 'M12 3v18M3 12h18M5 3h14a2 2 0 012 2v14a2 2 0 01-2 2H5a2 2 0 01-2-2V5a2 2 0 012-2z'
    }
  ]);

  // Session tab right-click menu + inline rename + group assignment
  let tabMenu = $state<{ tabId: string; x: number; y: number } | null>(null);
  let renamingTabId = $state<string | null>(null);
  let renameDraft = $state('');
  let isGroupModalOpen = $state(false);
  let groupingTabId = $state<string | null>(null);
  let groupDraft = $state('');
  let groupColorDraft = $state(GROUP_COLORS[0]);

  function openTabMenu(e: MouseEvent, tabId: string) {
    e.preventDefault();
    // Keep the menu inside the viewport on narrow windows.
    tabMenu = { tabId, x: Math.min(e.clientX, window.innerWidth - 176), y: e.clientY };
  }

  function startRename(tabId: string) {
    const tab = sessionTabs.find((item) => item.id === tabId);
    tabMenu = null;
    if (!tab) return;
    renameDraft = tabLabel(tab);
    renamingTabId = tabId;
  }

  function commitRename() {
    if (renamingTabId) renameTab(renamingTabId, renameDraft);
    renamingTabId = null;
  }

  function openGroupModal(tabId: string) {
    const tab = sessionTabs.find((item) => item.id === tabId);
    tabMenu = null;
    if (!tab) return;
    groupingTabId = tabId;
    groupDraft = tab.group || '';
    groupColorDraft = tab.groupColor || GROUP_COLORS[0];
    isGroupModalOpen = true;
  }

  function commitGroup() {
    if (groupingTabId) {
      setTabGroup(groupingTabId, groupDraft, groupDraft.trim() ? groupColorDraft : undefined);
    }
    isGroupModalOpen = false;
    groupingTabId = null;
  }

  function removeGroup(tabId: string) {
    tabMenu = null;
    setTabGroup(tabId, undefined, undefined);
  }

  function focusAndSelect(node: HTMLInputElement) {
    // Next frame: bind:value fills the input after actions run, and select() needs the text.
    requestAnimationFrame(() => {
      node.focus();
      node.select();
    });
  }

  /** A session tab is "current" only while the Session route is showing it. */
  function isSessionTabActive(tabId: string): boolean {
    if (!page.url.pathname.startsWith('/session')) return false;
    const selected = view.selectedTabId || sessionTabs[0]?.id;
    return selected === tabId;
  }

  // Files and AI share the right-hand column, so opening one closes the other.
  function handleFilesToggle() {
    if (view.showFiles) {
      setShowFiles(false);
      return;
    }
    closeAiChat();
    setShowFiles(true);
  }

  function handleAiToggle() {
    if (aiChat.open) {
      closeAiChat();
      return;
    }
    setShowFiles(false);
    toggleAiChat();
  }

  // Sidebar collapse & responsive mobile drawer state
  let isCollapsed = $state(false);
  let prevTabsCount = $state(0);
  let mobileDrawerOpen = $state(false);

  // Auto-collapse the sidebar while sessions are open. (Memory is handled natively now: the
  // shell sets WebView2's memory target to Low whenever CATerm is in the background.)
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

  /**
   * Lock keeps open session tabs (they reconnect after unlock); sign out also closes them.
   * Both drop the vault key from backend memory, not just the UI.
   */
  function lockApp() {
    closePalette();
    isUnlocked = false;
    onVaultLocked();
    lockVault().catch((err) => console.warn('lock_vault failed:', err));
  }

  async function signOut() {
  	const confirmed = await confirmModal(
  		t('shell.signOutConfirm'),
  		t('shell.signOutTitle'),
  		true,
  		t('profileMenu.signOut'),
  		t('common.cancel')
  	);
  	if (!confirmed) return;

  	closePalette();
  	closeAllTabs();
  	isUnlocked = false;
  	onVaultLocked();
  	await goto('/');
  	lockVault().catch((err) => console.warn('lock_vault failed:', err));
  	showToast(t('shell.signedOut'), 'info');
  }

  // ---- Keyboard shortcuts (list and rules in $lib/shortcuts) -----------------------------------
  const palette = getPalette();

  function currentTabIndex(): number {
    const index = sessionTabs.findIndex((tab) => tab.id === view.selectedTabId);
    return index < 0 ? 0 : index;
  }

  function showTab(index: number) {
    const tab = sessionTabs[index];
    if (!tab) return;
    setSelectedTabId(tab.id);
    if (!page.url.pathname.startsWith('/session')) void goto('/session');
  }

  /** Runs the command; false when there is nothing to act on, so the key passes through. */
  function runCommand(cmd: AppCommand): boolean {
    const onSession = page.url.pathname.startsWith('/session');
    switch (cmd.type) {
      case 'palette':
        if (palette.open && palette.mode === 'all') closePalette();
        else openPalette('all');
        return true;
      case 'newSession':
        openPalette('hosts');
        return true;
      case 'settings':
        void goto('/settings');
        return true;
      case 'lock':
        lockApp();
        return true;
      case 'tab': {
        if (sessionTabs.length === 0) return false;
        showTab(cmd.index < 0 ? sessionTabs.length - 1 : cmd.index);
        return true;
      }
      case 'cycleTab': {
        if (sessionTabs.length === 0) return false;
        // Away from the terminal the first press just brings the current session back.
        const next = onSession ? (currentTabIndex() + cmd.step + sessionTabs.length) % sessionTabs.length : currentTabIndex();
        showTab(next);
        return true;
      }
      case 'closeTab': {
        if (!onSession || sessionTabs.length === 0) return false;
        closeSessionTab(sessionTabs[currentTabIndex()].id);
        return true;
      }
    }
  }

  function handleShortcut(e: KeyboardEvent) {
    if (!isUnlocked) return;
    const cmd = appCommandFor(e);
    if (!cmd || !runCommand(cmd)) return;
    e.preventDefault();
    e.stopPropagation();
  }

  // $derived (not a plain const): labels re-resolve through t() whenever the locale changes.
  const navItems = $derived(getNavItems());
  const settingsItem = $derived(settingsNavItem());

  /** The page the title-bar chip names. From a terminal it points back to Hosts. */
  const currentSection = $derived(
    page.url.pathname.startsWith('/session')
      ? navItems[0]
      : ([...navItems, settingsItem].find((item) => isActive(item.href)) ?? navItems[0])
  );
</script>

<svelte:window
  onkeydown={(e) => {
    if (e.key !== 'Escape') return;
    mobileDrawerOpen = false;
    tabMenu = null;
    closeAiChat();
  }}
  onpointerdown={(e) => {
    if (tabMenu && !(e.target as HTMLElement | null)?.closest('[data-tab-menu]')) tabMenu = null;
  }}
  onresize={() => (tabMenu = null)}
/>

<NotificationCenter />
{#if isUnlocked}
  <CommandPalette onLock={lockApp} />
{/if}

{#if !isUnlocked}
  <LockScreen onUnlocked={() => { isUnlocked = true; void onVaultUnlocked(); }} />
{:else}
<!-- Shell: one title bar across the full width, then sidebar + content panel. Title bar and
     sidebar share the app background with no dividers; the content sits on a raised panel with
     a left/top hairline and a rounded top-left corner. -->
<div class="flex flex-col h-screen bg-neutral-100 dark:bg-[#0e0e0e] text-neutral-800 dark:text-neutral-300 font-sans transition-colors duration-150 pb-[env(safe-area-inset-bottom,0px)]">
  <!-- Title bar. Also hosts the terminal workspace controls (session tabs, Files, AI, split)
       so a session never costs a second stacked header row. -->
  <header
    class="min-h-[3rem] h-[calc(3rem+env(safe-area-inset-top,0px))] pt-[env(safe-area-inset-top,0px)] flex items-center gap-1 md:gap-2 pl-2 pr-1 md:pl-3 shrink-0 select-none cursor-default"
    data-tauri-drag-region
    onmousedown={startDragging}
    ondblclick={(e) => {
      const target = e.target as HTMLElement | null;
      if (!target?.closest('button, input, textarea, a, select, [role="button"], .no-drag')) {
        maximizeWindow();
      }
    }}
  >
    <!-- Left: current section + session tabs -->
    <div class="flex items-center gap-1 min-w-0 flex-1" data-tauri-drag-region>
      <button
        type="button"
        onclick={() => mobileDrawerOpen = true}
        class="md:hidden p-1 rounded text-neutral-600 dark:text-neutral-300 hover:text-neutral-900 dark:hover:text-white hover:bg-neutral-200/70 dark:hover:bg-neutral-800 transition-colors shrink-0"
        title={t('shell.openNav')}
        aria-label={t('shell.openNav')}
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
        </svg>
      </button>

      <div class="flex gap-1 text-xs items-center overflow-x-auto scrollbar-none min-w-0 flex-1" data-tauri-drag-region>
        <!-- Where you are, like a breadcrumb. Hidden while a terminal is showing: there the
             session tabs are the context. -->
        {#if !page.url.pathname.startsWith('/session')}
          <span class="px-2 py-1 text-xs font-semibold shrink-0 hidden sm:flex items-center gap-1.5 text-neutral-900 dark:text-white" data-tauri-drag-region>
            <svg class="w-3.5 h-3.5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={currentSection.path} />
            </svg>
            {currentSection.label}
          </span>
        {/if}
        {#each sessionTabs as tab (tab.id)}
          <!-- Hidden on phones: the Session route renders its own full-width tab switcher
               there, and two competing strips in a 375px row leaves both unusable. -->
          <div
            role="group"
            aria-label={t('shell.sessionTabAria', { label: tabLabel(tab) })}
            oncontextmenu={(e) => openTabMenu(e, tab.id)}
            class="hidden sm:flex items-center rounded-md shrink-0 transition-all border {isSessionTabActive(tab.id) ? 'bg-neutral-200 dark:bg-neutral-800 text-neutral-900 dark:text-white border-neutral-300 dark:border-neutral-700 shadow-2xs' : 'text-neutral-600 dark:text-neutral-400 hover:bg-neutral-200/70 dark:hover:bg-neutral-800/60 border-transparent'}"
          >
            {#if renamingTabId === tab.id}
              <span class="py-0.5 pl-2 pr-1 flex items-center gap-1.5">
                <span class="w-1.5 h-1.5 rounded-full shrink-0" style="background-color: {tab.groupColor || '#10b981'}"></span>
                <input
                  use:focusAndSelect
                  bind:value={renameDraft}
                  maxlength="40"
                  aria-label={t('shell.renameTab')}
                  onkeydown={(e) => {
                    if (e.key === 'Enter') commitRename();
                    else if (e.key === 'Escape') { e.stopPropagation(); renamingTabId = null; }
                  }}
                  onblur={commitRename}
                  class="w-28 md:w-36 px-1 py-0.5 rounded bg-white dark:bg-neutral-950 border border-sky-500 text-xs text-neutral-900 dark:text-white focus:outline-none"
                />
              </span>
            {:else}
              <a
                href="/session"
                onclick={() => setSelectedTabId(tab.id)}
                ondblclick={() => startRename(tab.id)}
                class="py-1 pl-2 pr-1 flex items-center gap-1.5 truncate max-w-[120px] md:max-w-[170px]"
                title={tab.group ? `[${tab.group}] ` + t('shell.tabTitle', { label: tabLabel(tab), address: tab.host.address }) : t('shell.tabTitle', { label: tabLabel(tab), address: tab.host.address })}
              >
                <span class="w-1.5 h-1.5 rounded-full shrink-0" style="background-color: {tab.groupColor || '#10b981'}"></span>
                <span class="truncate">{tabLabel(tab)}</span>
                {#if tab.group}
                  <span
                    class="px-1 py-0.1 rounded text-[9px] font-medium shrink-0 leading-tight"
                    style="background-color: {tab.groupColor ? tab.groupColor + '25' : 'rgba(2, 132, 199, 0.15)'}; color: {tab.groupColor || '#0284c7'}"
                  >
                    {tab.group}
                  </span>
                {/if}
              </a>
            {/if}
            <button
              onclick={() => closeSessionTab(tab.id)}
              class="p-0.5 mr-1 rounded hover:bg-neutral-300 dark:hover:bg-neutral-700 hover:text-rose-600 dark:hover:text-rose-400"
              title={t('shell.closeSessionNamed', { label: tabLabel(tab) })}
              aria-label={t('shell.closeSessionNamed', { label: tabLabel(tab) })}
            >
              <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-width="2" stroke-linecap="round" d="M6 18L18 6M6 6l12 12"></path></svg>
            </button>
          </div>
        {/each}
      </div>

      <!-- New session: the Hosts page is where a connection is actually picked. Kept outside
           the scrolling tab strip so it stays reachable once the tabs overflow. -->
      <a
        href="/"
        class="p-1 rounded shrink-0 text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white hover:bg-neutral-200/70 dark:hover:bg-neutral-800 transition-colors"
        title={t('shell.newSessionTitle')}
        aria-label={t('shell.newSession')}
      >
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"></path></svg>
      </a>

      <!-- Draggable blank space spanning remaining left area -->
      <div class="flex-1 h-full min-w-[20px]" data-tauri-drag-region></div>
    </div>

    <div class="flex items-center gap-1.5 text-neutral-500 dark:text-neutral-400 shrink-0" data-tauri-drag-region>
      <!-- Workspaces Menu -->
      <WorkspaceMenu />

      <!-- Notification Bell -->
      <button onclick={() => showToast(t('shell.noNotifications'), 'info')} class="p-1.5 hover:text-neutral-900 dark:hover:text-white hover:bg-neutral-200/70 dark:hover:bg-neutral-800 rounded-md transition-colors relative" title={t('shell.notifications')} aria-label={t('shell.notifications')}>
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9"></path></svg>
        {#if unreadCount > 0}
          <span class="absolute top-0.5 right-0.5 w-2 h-2 bg-rose-500 rounded-full border border-white dark:border-[#0e0e0e] animate-pulse"></span>
        {/if}
      </button>

      <!-- Unified Header Preferences: Theme, Language, Split View & Panels in 1 Elegant Icon -->
      <HeaderQuickControls
        layout={view.layout}
        showFiles={view.showFiles}
        aiOpen={aiChat.open}
        sessionCount={sessionTabs.length}
        {splitOptions}
        onSetLayout={setLayout}
        onToggleFiles={handleFilesToggle}
        onToggleAi={handleAiToggle}
      />

      <!-- Live Status Indicator Dot -->
      <div class="hidden sm:flex items-center text-xs" data-tauri-drag-region title={timeAgo}>
        <span class="text-emerald-500 animate-pulse text-[10px] p-1" class:opacity-50={monitorState.isPolling}>●</span>
      </div>

      <!-- Custom Window Controls -->
      <div class="hidden sm:flex items-center no-drag">
        <button onclick={() => minimizeWindow()} class="p-2 rounded-md hover:bg-neutral-200/70 dark:hover:bg-neutral-800 text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white transition-colors" title={t('shell.minimize')} aria-label={t('shell.minimize')}>
          <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 12H4"></path></svg>
        </button>
        <button onclick={() => maximizeWindow()} class="p-2 rounded-md hover:bg-neutral-200/70 dark:hover:bg-neutral-800 text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white transition-colors" title={t('shell.maximize')} aria-label={t('shell.maximize')}>
          <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><rect x="4" y="4" width="16" height="16" rx="2" stroke-width="2"></rect></svg>
        </button>
        <button onclick={() => closeWindow()} class="p-2 rounded-md hover:bg-rose-500 hover:text-white text-neutral-500 dark:text-neutral-400 transition-colors" title={t('shell.close')} aria-label={t('shell.close')}>
          <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path></svg>
        </button>
      </div>
    </div>
  </header>

  <div class="flex flex-1 min-h-0">
    <!-- Mobile Slide-out Drawer Backdrop -->
    {#if mobileDrawerOpen}
      <button
        type="button"
        onclick={() => mobileDrawerOpen = false}
        class="fixed inset-0 z-40 bg-black/60 backdrop-blur-sm md:hidden transition-opacity border-0 p-0 cursor-default"
        aria-label={t('shell.closeMenuBackdrop')}
      ></button>
    {/if}

    <!-- Mobile Slide-out Drawer Navigation -->
    <aside
      class="fixed inset-y-0 left-0 z-50 w-72 max-w-[85vw] bg-neutral-100 dark:bg-[#0e0e0e] border-r border-neutral-200 dark:border-neutral-800 flex flex-col justify-between shadow-2xl md:hidden transform transition-transform duration-200 ease-in-out {mobileDrawerOpen ? 'translate-x-0' : '-translate-x-full'}"
      aria-label={t('shell.mobileNav')}
    >
      <div class="min-h-0 flex flex-col">
        <div class="min-h-[3rem] h-[calc(3rem+env(safe-area-inset-top,0px))] pt-[env(safe-area-inset-top,0px)] flex items-center justify-between px-4">
          <div class="flex items-center gap-2">
            <Logo size={22} mode="brand" />
            <span class="font-bold text-neutral-900 dark:text-white text-base tracking-tight">CATerm</span>
            <span class="text-[10px] px-1.5 py-0.5 rounded border border-neutral-300 dark:border-neutral-700 text-neutral-500 dark:text-neutral-400 font-mono">v{APP_VERSION}</span>
          </div>
          <button
            onclick={() => mobileDrawerOpen = false}
            class="p-1.5 rounded-lg hover:bg-neutral-200/70 dark:hover:bg-neutral-800 text-neutral-500 hover:text-neutral-900 dark:text-neutral-400 dark:hover:text-white transition-colors"
            aria-label={t('shell.closeNav')}
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <p class="px-5 pt-2 pb-1.5 text-[10px] font-semibold uppercase tracking-[0.12em] text-neutral-500">{t('shell.workspace')}</p>
        <nav class="px-2 space-y-0.5 overflow-y-auto scrollbar-none text-sm">
          {#each navItems as item}
            <a
              href={item.href}
              onclick={() => mobileDrawerOpen = false}
              title={item.label}
              aria-current={isActive(item.href) ? 'page' : undefined}
              class="px-2.5 py-2 rounded-lg flex items-center gap-3 transition-colors {isActive(item.href) ? 'bg-neutral-200/80 dark:bg-neutral-800/80 text-neutral-900 dark:text-white font-medium' : 'text-neutral-600 dark:text-neutral-400 hover:bg-neutral-200/50 dark:hover:bg-neutral-800/40 hover:text-neutral-900 dark:hover:text-white'}"
            >
              <svg class="w-[18px] h-[18px] shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d={item.path} />
              </svg>
              <span class="truncate">{item.label}</span>
            </a>
          {/each}
        </nav>
      </div>

      <div class="p-2 pb-[max(0.75rem,env(safe-area-inset-bottom,0px))]">
        <ProfileMenu onLock={lockApp} onSignOut={signOut} onNavigate={() => (mobileDrawerOpen = false)} />
      </div>
    </aside>

    <!-- Desktop Sidebar -->
    <aside
      class="hidden md:flex flex-col shrink-0 will-change-[width] {isCollapsed ? 'w-16' : 'w-60'}"
      aria-label={t('shell.sidebar')}
    >
      <!-- Brand + collapse -->
      <div class="flex items-center pt-3 pb-4 {isCollapsed ? 'justify-center px-2' : 'justify-between pl-4 pr-2'}">
        {#if isCollapsed}
          <button
            type="button"
            onclick={toggleSidebar}
            title={t('shell.expandSidebar')}
            aria-label={t('shell.expandSidebar')}
            class="p-1.5 rounded-lg hover:bg-neutral-200/70 dark:hover:bg-neutral-800 transition-colors"
          >
            <Logo size={22} mode="brand" />
          </button>
        {:else}
          <div class="flex items-center gap-2 min-w-0">
            <Logo size={22} mode="brand" />
            <span class="font-bold text-neutral-900 dark:text-white text-lg tracking-tight truncate">CATerm</span>
            <span class="text-[10px] px-1.5 py-0.5 rounded border border-neutral-300 dark:border-neutral-700 text-neutral-500 dark:text-neutral-400 font-mono shrink-0">v{APP_VERSION}</span>
          </div>
          <button
            type="button"
            onclick={toggleSidebar}
            title={t('shell.collapseSidebar')}
            aria-label={t('shell.collapseSidebar')}
            class="p-1 rounded-md hover:bg-neutral-200/70 dark:hover:bg-neutral-800 text-neutral-400 hover:text-neutral-700 dark:hover:text-neutral-200 transition-colors shrink-0"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 19l-7-7 7-7m8 14l-7-7 7-7" />
            </svg>
          </button>
        {/if}
      </div>

      {#if !isCollapsed}
        <p class="px-5 pb-1.5 text-[10px] font-semibold uppercase tracking-[0.12em] text-neutral-500">{t('shell.workspace')}</p>
      {/if}
      <nav class="flex-1 min-h-0 overflow-y-auto scrollbar-none px-2 space-y-0.5 text-sm">
        {#each navItems as item}
          <a
            href={item.href}
            title={item.label}
            aria-current={isActive(item.href) ? 'page' : undefined}
            class="relative px-2.5 py-2 rounded-lg flex items-center {isCollapsed ? 'justify-center' : 'gap-3'} transition-colors {isActive(item.href) ? 'bg-neutral-200/80 dark:bg-neutral-800/80 text-neutral-900 dark:text-white font-medium' : 'text-neutral-600 dark:text-neutral-400 hover:bg-neutral-200/50 dark:hover:bg-neutral-800/40 hover:text-neutral-900 dark:hover:text-white'}"
          >
            {#if isActive(item.href)}
              <!-- Active marker pinned to the window's left edge -->
              <span class="absolute -left-2 top-1.5 bottom-1.5 w-[3px] rounded-r bg-neutral-900 dark:bg-white" aria-hidden="true"></span>
            {/if}
            <svg class="w-[18px] h-[18px] shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d={item.path} />
            </svg>
            {#if !isCollapsed}
              <span class="truncate">{item.label}</span>
            {/if}
          </a>
        {/each}
      </nav>

      <div class="p-2">
        <ProfileMenu collapsed={isCollapsed} onLock={lockApp} onSignOut={signOut} />
      </div>
    </aside>

    <!-- Content panel. The AI panel is a docked column beside the page rather than an overlay,
         so it behaves like the Files panel: the content narrows instead of being covered. Both
         never show at once — see handleAiToggle. -->
    <main class="flex-1 min-w-0 flex overflow-hidden relative bg-white dark:bg-[#161616] border-t border-neutral-200 dark:border-neutral-800 md:border-l md:rounded-tl-xl transition-colors duration-150">
      <!-- Session Viewport: fleksibel agar menyisakan ruang untuk AI panel -->
      <div class="flex-1 min-w-0 flex {page.url.pathname.startsWith('/session') ? 'opacity-100 z-10 pointer-events-auto' : 'opacity-0 pointer-events-none -z-10 absolute inset-0 invisible'}">
        <SessionViewport />
      </div>

      <!-- Other pages routed via SvelteKit children -->
      <div class="flex-1 min-w-0 overflow-auto text-neutral-900 dark:text-neutral-100 relative px-4 py-6 md:px-10 md:py-10 [@media(max-height:500px)]:py-4 bg-[linear-gradient(to_right,#0000000a_1px,transparent_1px),linear-gradient(to_bottom,#0000000a_1px,transparent_1px)] dark:bg-[linear-gradient(to_right,#ffffff08_1px,transparent_1px),linear-gradient(to_bottom,#ffffff08_1px,transparent_1px)] bg-[size:56px_56px] {page.url.pathname.startsWith('/session') ? 'hidden' : 'z-20'}">
        {@render children()}
      </div>

      <div class={['contents', !aiChat.open && 'hidden'].filter(Boolean).join(' ')}>
        <AiChatPanel onClose={closeAiChat} />
      </div>

      {#if feedbackPrompt.show}
        <!-- Feedback popup modal: auto-prompted after the 3rd closed session, or opened from the
             profile menu (Report bug). Modal popup like AboutModal. -->
        <FeedbackModal
          onClose={() => feedbackPrompt.close()}
          onSubmitted={() => feedbackPrompt.markSubmitted()}
        />
      {/if}

      {#if pendingCrashReport}
        <CrashReportModal
          report={pendingCrashReport}
          onClose={() => (pendingCrashReport = null)}
        />
      {/if}
    </main>
  </div>
</div>

{#if tabMenu}
  {@const menuTab = sessionTabs.find((item) => item.id === tabMenu?.tabId)}
  {#if menuTab}
    <div
      data-tab-menu
      role="menu"
      aria-label={t('shell.sessionTabMenu')}
      class="fixed z-[200] w-48 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-[#141414] shadow-2xl py-1 text-sm"
      style="left: {tabMenu.x}px; top: {tabMenu.y}px;"
    >
      <button
        role="menuitem"
        onclick={() => startRename(menuTab.id)}
        class="w-full flex items-center gap-2.5 px-3 py-1.5 text-left text-neutral-700 dark:text-neutral-200 hover:bg-neutral-100 dark:hover:bg-neutral-800/60 transition-colors cursor-pointer"
      >
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M16.9 4.1a2.1 2.1 0 013 3L8.5 18.5 4 20l1.5-4.5z" /></svg>
        {t('shell.rename')}
      </button>

      <button
        role="menuitem"
        onclick={() => openGroupModal(menuTab.id)}
        class="w-full flex items-center gap-2.5 px-3 py-1.5 text-left text-neutral-700 dark:text-neutral-200 hover:bg-neutral-100 dark:hover:bg-neutral-800/60 transition-colors cursor-pointer"
      >
        <svg class="w-3.5 h-3.5 text-sky-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M7 7h.01M7 3h5c.512 0 1.024.195 1.414.586l7 7a2 2 0 010 2.828l-7 7a2 2 0 01-2.828 0l-7-7A1.994 1.994 0 013 12V7a4 4 0 014-4z" />
        </svg>
        {t('shell.setGroup')}
      </button>

      {#if menuTab.group}
        <button
          role="menuitem"
          onclick={() => removeGroup(menuTab.id)}
          class="w-full flex items-center gap-2.5 px-3 py-1.5 text-left text-neutral-500 dark:text-neutral-400 hover:bg-neutral-100 dark:hover:bg-neutral-800/60 transition-colors cursor-pointer text-xs"
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M6 18L18 6M6 6l12 12" /></svg>
          {t('shell.removeGroup')}
        </button>

        <button
          role="menuitem"
          onclick={() => { const grp = menuTab.group; tabMenu = null; if (grp) closeTabsInGroup(grp); }}
          class="w-full flex items-center gap-2.5 px-3 py-1.5 text-left text-amber-600 dark:text-amber-400 hover:bg-amber-50 dark:hover:bg-amber-950/30 transition-colors cursor-pointer text-xs"
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" /></svg>
          {t('shell.closeGroup', { group: menuTab.group })}
        </button>
      {/if}

      <div class="my-1 border-t border-neutral-200 dark:border-neutral-800"></div>

      <button
        role="menuitem"
        onclick={() => { const id = menuTab.id; tabMenu = null; closeSessionTab(id); }}
        class="w-full flex items-center gap-2.5 px-3 py-1.5 text-left text-rose-600 dark:text-rose-400 hover:bg-rose-50 dark:hover:bg-rose-950/30 transition-colors cursor-pointer"
      >
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-width="1.8" d="M6 18L18 6M6 6l12 12" /></svg>
        {t('shell.closeSession')}
      </button>
    </div>
  {/if}
{/if}

<!-- Tab Grouping Modal -->
{#if isGroupModalOpen}
  <div
    class="fixed inset-0 z-[99999] bg-black/40 dark:bg-black/60 backdrop-blur-xs flex items-center justify-center p-4 animate-in fade-in duration-150"
  >
    <div
      class="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-2xl p-5 max-w-sm w-full shadow-2xl space-y-4 animate-in zoom-in-95 duration-150"
      role="dialog"
      aria-modal="true"
    >
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2">
          <div class="w-7 h-7 rounded-lg bg-sky-500/10 dark:bg-sky-500/20 text-sky-600 dark:text-sky-400 flex items-center justify-center">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 7h.01M7 3h5c.512 0 1.024.195 1.414.586l7 7a2 2 0 010 2.828l-7 7a2 2 0 01-2.828 0l-7-7A1.994 1.994 0 013 12V7a4 4 0 014-4z" />
            </svg>
          </div>
          <h3 class="font-bold text-neutral-900 dark:text-white text-sm">{t('shell.setGroup')}</h3>
        </div>
        <button
          type="button"
          onclick={() => isGroupModalOpen = false}
          aria-label={t('common.cancel')}
          class="p-1 rounded-lg text-neutral-400 hover:text-neutral-600 dark:hover:text-white hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>
        </button>
      </div>

      <div class="space-y-3">
        <div>
          <label for="tab-group-name-input" class="block text-xs font-medium text-neutral-700 dark:text-neutral-300 mb-1">
            Group Name
          </label>
          <input
            id="tab-group-name-input"
            type="text"
            bind:value={groupDraft}
            placeholder="e.g. Production, Backend, Database"
            maxlength="24"
            class="w-full bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded-lg px-3 py-1.5 text-xs text-neutral-900 dark:text-white placeholder-neutral-400 dark:placeholder-neutral-500 focus:outline-none focus:border-sky-500 transition-colors"
            onkeydown={(e) => e.key === 'Enter' && commitGroup()}
          />
        </div>

        <div>
          <span class="block text-xs font-medium text-neutral-700 dark:text-neutral-300 mb-1.5">
            Group Color
          </span>
          <div class="flex items-center gap-2">
            {#each GROUP_COLORS as color}
              <button
                type="button"
                onclick={() => groupColorDraft = color}
                class="w-6 h-6 rounded-full transition-transform flex items-center justify-center {groupColorDraft === color ? 'scale-115 ring-2 ring-offset-2 ring-neutral-400 dark:ring-neutral-500' : 'hover:scale-105'}"
                style="background-color: {color};"
                aria-label={color}
              >
                {#if groupColorDraft === color}
                  <svg class="w-3.5 h-3.5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M5 13l4 4L19 7" /></svg>
                {/if}
              </button>
            {/each}
          </div>
        </div>
      </div>

      <div class="flex justify-end gap-2 pt-2 border-t border-neutral-100 dark:border-neutral-800">
        {#if groupingTabId && sessionTabs.find(t => t.id === groupingTabId)?.group}
          <button
            type="button"
            onclick={() => { if (groupingTabId) removeGroup(groupingTabId); isGroupModalOpen = false; }}
            class="mr-auto px-2.5 py-1 text-rose-600 dark:text-rose-400 hover:bg-rose-50 dark:hover:bg-rose-950/30 rounded-lg text-xs font-medium transition-colors"
          >
            {t('common.remove')}
          </button>
        {/if}
        <button
          type="button"
          onclick={() => isGroupModalOpen = false}
          class="px-3 py-1.5 bg-neutral-100 hover:bg-neutral-200 dark:bg-neutral-800 dark:hover:bg-neutral-700 text-neutral-700 dark:text-neutral-300 rounded-lg text-xs font-medium transition-colors cursor-pointer"
        >
          {t('common.cancel')}
        </button>
        <button
          type="button"
          onclick={commitGroup}
          class="px-4 py-1.5 bg-sky-600 hover:bg-sky-500 text-white rounded-lg text-xs font-medium transition-colors shadow-md shadow-sky-600/20 cursor-pointer"
        >
          {t('common.save')}
        </button>
      </div>
    </div>
  </div>
{/if}
{/if}
