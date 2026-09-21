<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import TerminalPane from '$lib/components/TerminalPane.svelte';
  import SessionFileManager from '$lib/components/SessionFileManager.svelte';
  import { listHosts } from '$lib/api/hosts';
  import { getTabs, openTab, closeTab, tabLabel } from '$lib/stores/sessionTabs.svelte';
  import {
    getSessionView,
    setSelectedTabId,
    setShowFiles,
    setLayout,
    resetLayout
  } from '$lib/stores/sessionView.svelte';

  // Layout, file-panel visibility and the selected host are driven from the app header now
  // (single row — see routes/+layout.svelte), so they live in a shared store instead of here.
  const view = getSessionView();
  let loadError = $state('');

  // Split layouts are a desktop affordance; phones always show one terminal at a time. This is
  // tracked in JS rather than with `sm:` classes because rendering a separate mobile tree would
  // mount a *second* TerminalPane per host — and a display:none component still runs onMount,
  // so every host would open two SSH sessions.
  let isWideViewport = $state(true);

  onMount(() => {
    const query = window.matchMedia('(min-width: 640px)');
    const sync = () => (isWideViewport = query.matches);
    sync();
    query.addEventListener('change', sync);
    return () => query.removeEventListener('change', sync);
  });

  // Opens a tab for every host named in `?host=<id>` or `?hosts=<id1>,<id2>,...` — this is
  // the ONLY way a session tab gets created. No fallback/demo hosts: with no matching query
  // param, or before the id resolves, nothing opens (see the empty state below).
  //
  // Each distinct URL is honoured exactly once. `openSession()` stamps a nonce onto every
  // Connect so asking for a host that is already open is a new URL, and therefore opens a
  // second session rather than being swallowed as a duplicate.
  let handledUrl = '';

  $effect(() => {
    const url = page.url.href;
    const params = page.url.searchParams;
    if (url === handledUrl) return;

    const ids: string[] = [];
    const single = params.get('host');
    if (single) ids.push(single);
    const multi = params.get('hosts');
    if (multi) {
      for (const id of multi.split(',').map((s) => s.trim()).filter(Boolean)) ids.push(id);
    }
    if (ids.length === 0) return;

    handledUrl = url;

    listHosts()
      .then((hosts) => {
        loadError = '';
        let lastOpened = '';
        for (const id of ids) {
          const host = hosts.find((h) => h.id === id);
          if (host) lastOpened = openTab(host);
        }
        // Focus what was just opened, so a second session to the same host is visible at once.
        if (lastOpened) setSelectedTabId(lastOpened);
      })
      .catch(() => {
        loadError = 'Tauri backend tidak terdeteksi — tidak bisa memuat daftar host.';
      });
  });

  const tabs = $derived(getTabs());

  $effect(() => {
    if (tabs.length > 0 && (!view.selectedTabId || !tabs.some((t) => t.id === view.selectedTabId))) {
      setSelectedTabId(tabs[0].id);
    }
  });

  const activeTab = $derived.by(() => {
    if (tabs.length === 0) return undefined;
    return tabs.find((t) => t.id === view.selectedTabId) ?? tabs[0];
  });

  const activeHost = $derived(activeTab?.host);

  /** A single pane is the only sensible arrangement on a phone, or with one host open. */
  const effectiveLayout = $derived(!isWideViewport || tabs.length === 1 ? 1 : view.layout);

  const containerClass = $derived(
    effectiveLayout === 2
      ? 'flex flex-col h-full gap-1 sm:gap-1.5'
      : effectiveLayout === 3
        ? 'flex h-full gap-1 sm:gap-1.5'
        : effectiveLayout === 4
          ? 'grid grid-cols-2 auto-rows-fr h-full gap-1 sm:gap-1.5'
          : 'relative h-full w-full'
  );

  /**
   * In the stacked (single-pane) arrangement every tab stays mounted and only the selected one
   * is shown, so switching tabs never tears down an SSH session or loses scrollback. Hiding is
   * done with `invisible` rather than `hidden`: `display:none` collapses the box to 0x0 and
   * xterm would re-measure itself to nothing.
   */
  function paneClass(tabId: string): string {
    if (effectiveLayout !== 1) return 'min-h-0 min-w-0 flex-1';
    return tabId === activeTab?.id
      ? 'absolute inset-0 z-10'
      : 'absolute inset-0 invisible pointer-events-none';
  }

  function close(tabId: string) {
    closeTab(tabId);
    if (getTabs().length === 0) resetLayout();
    if (view.selectedTabId === tabId) {
      setSelectedTabId(getTabs()[0]?.id || '');
    }
  }
</script>

<div class="h-full w-full flex flex-col bg-neutral-100 dark:bg-neutral-950 overflow-hidden">
  <!-- Terminal & Content Viewport Area. No header bar here on purpose: host, Files toggle and
       split controls all live in the app's single top header, and each pane carries its own
       host/address strip. -->
  <div class="flex-1 w-full min-h-0 overflow-hidden p-1 sm:p-1.5">
    {#if loadError}
      <div class="p-4 text-sm text-amber-400 bg-neutral-900 border border-neutral-800 rounded">{loadError}</div>
    {:else if tabs.length === 0}
      <div class="h-full flex flex-col items-center justify-center gap-3 text-center border border-dashed border-neutral-800 rounded-lg p-4">
        <p class="text-neutral-300 font-medium">Belum ada sesi terminal yang dibuka.</p>
        <p class="text-neutral-500 text-sm max-w-sm">Pilih host dari halaman Hosts (atau Launch All dari sebuah Group) untuk membuka sesi SSH di sini.</p>
        <a href="/" class="px-4 py-2 bg-sky-600 hover:bg-sky-500 text-white text-sm font-medium rounded-md transition-colors">Ke halaman Hosts</a>
      </div>
    {:else}
      <div class="flex h-full gap-1 sm:gap-1.5 overflow-hidden">
        <!-- Terminal Panes Section -->
        <div class="flex-1 min-w-0 h-full overflow-hidden flex flex-col">
          <!-- Mobile tab switcher (only when multi-host on a phone) -->
          {#if tabs.length > 1 && !isWideViewport}
            <div class="flex items-center gap-1 overflow-x-auto pb-1.5 mb-1 scrollbar-none shrink-0">
              {#each tabs as tab (tab.id)}
                <button
                  onclick={() => setSelectedTabId(tab.id)}
                  class="px-2.5 py-1 rounded text-xs font-mono transition-colors shrink-0 flex items-center gap-1.5 {view.selectedTabId === tab.id ? 'bg-sky-600 text-white font-semibold shadow-xs' : 'bg-neutral-900 text-neutral-400 hover:text-neutral-200 border border-neutral-800'}"
                >
                  <span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
                  <span class="truncate max-w-[120px]">{tabLabel(tab)}</span>
                </button>
              {/each}
            </div>
          {/if}

          <!-- One TerminalPane instance per open tab, keyed by host id and never rebuilt by a
               layout or tab change. Re-using a single instance across hosts is what made the
               XPC tab render YPC's live session: Svelte swapped the `host` prop but onMount —
               and therefore the xterm instance and the SSH session — stayed with the old host. -->
          <div class="flex-1 min-h-0">
            <div class={containerClass}>
              {#each tabs as tab (tab.id)}
                <div class={paneClass(tab.id)}>
                  <TerminalPane
                    host={tab.host}
                    label={tabLabel(tab)}
                    isActive={effectiveLayout === 1 ? tab.id === activeTab?.id : true}
                    onSplitRight={effectiveLayout === 1 && tabs.length > 1 ? () => setLayout(3) : undefined}
                    onSplitDown={effectiveLayout === 1 && tabs.length > 1 ? () => setLayout(2) : undefined}
                    onClose={() => close(tab.id)}
                  />
                </div>
              {/each}
            </div>
          </div>
        </div>

        <!-- Remote File Explorer Panel (SFTP) -->
        {#if view.showFiles && activeHost}
          <!-- Desktop: inline side panel -->
          <div class="hidden md:block w-80 lg:w-96 shrink-0 h-full overflow-hidden">
            <SessionFileManager
              host={activeHost}
              availableHosts={tabs.map((t) => t.host)}
              onSelectHost={(h) => setSelectedTabId(tabs.find((t) => t.host.id === h.id)?.id ?? '')}
              onClose={() => setShowFiles(false)}
            />
          </div>

          <!-- Mobile: full overlay drawer modal with close button -->
          <div class="md:hidden fixed inset-0 z-50 bg-black/70 backdrop-blur-xs flex flex-col justify-end p-2">
            <div class="w-full h-full max-h-[94vh] flex flex-col bg-neutral-950 rounded-lg shadow-2xl border border-neutral-800 overflow-hidden">
              <SessionFileManager
                host={activeHost}
                availableHosts={tabs.map((t) => t.host)}
                onSelectHost={(h) => setSelectedTabId(tabs.find((t) => t.host.id === h.id)?.id ?? '')}
                onClose={() => setShowFiles(false)}
              />
            </div>
          </div>
        {/if}
      </div>
    {/if}
  </div>
</div>
