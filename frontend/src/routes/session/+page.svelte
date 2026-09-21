<script lang="ts">
  import { page } from '$app/state';
  import TerminalPane from '$lib/components/TerminalPane.svelte';
  import SessionFileManager from '$lib/components/SessionFileManager.svelte';
  import { listHosts, type HostRecord } from '$lib/api/hosts';
  import { getTabs, openTab, closeTab } from '$lib/stores/sessionTabs.svelte';
  import {
    getSessionView,
    setSelectedHostId,
    setShowFiles,
    resetLayout
  } from '$lib/stores/sessionView.svelte';

  // Layout, file-panel visibility and the selected host are driven from the app header now
  // (single row — see routes/+layout.svelte), so they live in a shared store instead of here.
  const view = getSessionView();
  let loadError = $state('');

  // Opens a tab for every host named in `?host=<id>` or `?hosts=<id1>,<id2>,...` — this is
  // the ONLY way a session tab gets created. No fallback/demo hosts: with no matching query
  // param, or before the id resolves, nothing opens (see the empty state below).
  $effect(() => {
    const params = page.url.searchParams;
    const ids = new Set<string>();
    const single = params.get('host');
    if (single) ids.add(single);
    const multi = params.get('hosts');
    if (multi) {
      for (const id of multi.split(',').map((s) => s.trim()).filter(Boolean)) ids.add(id);
    }
    if (ids.size === 0) return;

    listHosts()
      .then((hosts) => {
        loadError = '';
        for (const id of ids) {
          const host = hosts.find((h) => h.id === id);
          if (host) openTab(host);
        }
      })
      .catch(() => {
        loadError = 'Tauri backend tidak terdeteksi — tidak bisa memuat daftar host.';
      });
  });

  const tabs = $derived(getTabs());
  const pane = (i: number): HostRecord | undefined => tabs[i]?.id !== undefined ? tabs[i].host : undefined;

  $effect(() => {
    if (tabs.length > 0 && (!view.selectedHostId || !tabs.some((t) => t.host.id === view.selectedHostId))) {
      setSelectedHostId(tabs[0].host.id);
    }
  });

  const activeHost = $derived.by(() => {
    if (tabs.length === 0) return undefined;
    if (view.selectedHostId) {
      const found = tabs.find((t) => t.host.id === view.selectedHostId);
      if (found) return found.host;
    }
    return tabs[0]?.host;
  });

  function close(host: HostRecord) {
    closeTab(host.id);
    if (getTabs().length === 0) resetLayout();
    if (view.selectedHostId === host.id) {
      setSelectedHostId(tabs[0]?.host.id || '');
    }
  }
</script>

<div class="h-full w-full flex flex-col bg-neutral-950 overflow-hidden">
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
        <!-- Terminal Panes Section (Responsive) -->
        <div class="flex-1 min-w-0 h-full overflow-hidden flex flex-col">
          <!-- Mobile tab switcher (only when multi-host on mobile) -->
          {#if tabs.length > 1}
            <div class="sm:hidden flex items-center gap-1 overflow-x-auto pb-1.5 mb-1 scrollbar-none shrink-0">
              {#each tabs as tab (tab.id)}
                <button
                  onclick={() => setSelectedHostId(tab.host.id)}
                  class="px-2.5 py-1 rounded text-xs font-mono transition-colors shrink-0 flex items-center gap-1.5 {view.selectedHostId === tab.host.id ? 'bg-sky-600 text-white font-semibold shadow-xs' : 'bg-neutral-900 text-neutral-400 hover:text-neutral-200 border border-neutral-800'}"
                >
                  <span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
                  <span class="truncate max-w-[120px]">{tab.host.label}</span>
                </button>
              {/each}
            </div>
          {/if}

          <!-- Mobile view: single clean full-viewport terminal for selected host -->
          <div class="sm:hidden flex-1 min-h-0">
            {#if activeHost}
              <TerminalPane host={activeHost} onClose={() => close(activeHost)} />
            {/if}
          </div>

          <!-- Desktop view: multi-pane layout options -->
          <div class="hidden sm:block flex-1 min-h-0 h-full">
            {#if view.layout === 1 || tabs.length === 1}
              {@const h = activeHost || pane(0)}
              {#if h}
                <TerminalPane
                  host={h}
                  onSplitRight={tabs.length > 1 ? () => (view.layout = 3) : undefined}
                  onSplitDown={tabs.length > 1 ? () => (view.layout = 2) : undefined}
                  onClose={() => close(h)}
                />
              {/if}
            {:else if view.layout === 2}
              <div class="flex flex-col h-full gap-1 sm:gap-1.5">
                {#each tabs as tab (tab.id)}
                  <div class="flex-1 min-h-0">
                    <TerminalPane host={tab.host} onClose={() => close(tab.host)} />
                  </div>
                {/each}
              </div>
            {:else if view.layout === 3}
              <div class="flex h-full gap-1 sm:gap-1.5">
                {#each tabs as tab (tab.id)}
                  <div class="flex-1 min-w-0">
                    <TerminalPane host={tab.host} onClose={() => close(tab.host)} />
                  </div>
                {/each}
              </div>
            {:else if view.layout === 4}
              <div class="grid grid-cols-2 grid-rows-2 h-full gap-1 sm:gap-1.5">
                {#each tabs.slice(0, 4) as tab (tab.id)}
                  <div class="min-h-0 min-w-0">
                    <TerminalPane host={tab.host} onClose={() => close(tab.host)} />
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        </div>

        <!-- Remote File Explorer Panel (SFTP) -->
        {#if view.showFiles && activeHost}
          <!-- Desktop: inline side panel -->
          <div class="hidden md:block w-80 lg:w-96 shrink-0 h-full overflow-hidden">
            <SessionFileManager
              host={activeHost}
              availableHosts={tabs.map((t) => t.host)}
              onSelectHost={(h) => setSelectedHostId(h.id)}
              onClose={() => setShowFiles(false)}
            />
          </div>

          <!-- Mobile: full overlay drawer modal with close button -->
          <div class="md:hidden fixed inset-0 z-50 bg-black/70 backdrop-blur-xs flex flex-col justify-end p-2">
            <div class="w-full h-full max-h-[94vh] flex flex-col bg-neutral-950 rounded-lg shadow-2xl border border-neutral-800 overflow-hidden">
              <SessionFileManager
                host={activeHost}
                availableHosts={tabs.map((t) => t.host)}
                onSelectHost={(h) => setSelectedHostId(h.id)}
                onClose={() => setShowFiles(false)}
              />
            </div>
          </div>
        {/if}
      </div>
    {/if}
  </div>
</div>
