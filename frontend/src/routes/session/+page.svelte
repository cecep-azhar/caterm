<script lang="ts">
  import { page } from '$app/state';
  import TerminalPane from '$lib/components/TerminalPane.svelte';
  import SessionFileManager from '$lib/components/SessionFileManager.svelte';
  import { listHosts, type HostRecord } from '$lib/api/hosts';
  import { getTabs, openTab, closeTab } from '$lib/stores/sessionTabs.svelte';

  // 1 = single, 2 = split horizontal, 3 = split vertical, 4 = grid 2x2
  let layout = $state(1);
  let loadError = $state('');
  let showFiles = $state(true);
  let selectedHostId = $state<string>('');

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

  const activeHost = $derived.by(() => {
    if (tabs.length === 0) return undefined;
    if (selectedHostId) {
      const found = tabs.find((t) => t.host.id === selectedHostId);
      if (found) return found.host;
    }
    return tabs[0]?.host;
  });

  function close(host: HostRecord) {
    closeTab(host.id);
    if (getTabs().length === 0) layout = 1;
    if (selectedHostId === host.id) {
      selectedHostId = tabs[0]?.host.id || '';
    }
  }
</script>

<div class="h-full flex flex-col space-y-2">
  <div class="flex justify-between items-center bg-neutral-900 border border-neutral-800 rounded px-4 py-2 shrink-0">
    <div class="flex items-center gap-3">
      <h1 class="text-sm font-bold text-white tracking-tight">Active Sessions</h1>
      {#if tabs.length > 0}
        <button
          onclick={() => showFiles = !showFiles}
          class="px-2.5 py-1 rounded text-xs font-medium border transition-colors flex items-center gap-1.5 {showFiles ? 'bg-sky-600/20 text-sky-400 border-sky-500/30 hover:bg-sky-600/30' : 'bg-neutral-950 text-neutral-400 border-neutral-800 hover:text-white hover:border-neutral-700'}"
          title={showFiles ? 'Hide Remote Files' : 'Show Remote Files'}
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"></path>
          </svg>
          <span>Files</span>
        </button>
      {/if}
    </div>

    {#if tabs.length > 0}
      <div class="flex gap-1 bg-neutral-950 p-1 rounded border border-neutral-800">
        <button onclick={() => layout = 1} class="p-1 rounded {layout === 1 ? 'bg-sky-600/20 text-sky-400' : 'text-neutral-500 hover:text-neutral-300'}" title="Single">
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><rect x="3" y="3" width="18" height="18" rx="2" stroke-width="2"></rect></svg>
        </button>
        <button onclick={() => layout = 2} disabled={tabs.length < 2} class="p-1 rounded disabled:opacity-30 {layout === 2 ? 'bg-sky-600/20 text-sky-400' : 'text-neutral-500 hover:text-neutral-300'}" title="Split Horizontal">
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-width="2" d="M3 12h18M5 3h14a2 2 0 012 2v14a2 2 0 01-2 2H5a2 2 0 01-2-2V5a2 2 0 012-2z"></path></svg>
        </button>
        <button onclick={() => layout = 3} disabled={tabs.length < 2} class="p-1 rounded disabled:opacity-30 {layout === 3 ? 'bg-sky-600/20 text-sky-400' : 'text-neutral-500 hover:text-neutral-300'}" title="Split Vertical">
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-width="2" d="M12 3v18M5 3h14a2 2 0 012 2v14a2 2 0 01-2 2H5a2 2 0 01-2-2V5a2 2 0 012-2z"></path></svg>
        </button>
        <button onclick={() => layout = 4} disabled={tabs.length < 3} class="p-1 rounded disabled:opacity-30 {layout === 4 ? 'bg-sky-600/20 text-sky-400' : 'text-neutral-500 hover:text-neutral-300'}" title="Grid 2x2">
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-width="2" d="M12 3v18M3 12h18M5 3h14a2 2 0 012 2v14a2 2 0 01-2 2H5a2 2 0 01-2-2V5a2 2 0 012-2z"></path></svg>
        </button>
      </div>
    {/if}
  </div>

  <div class="flex-1 w-full h-full overflow-hidden">
    {#if loadError}
      <div class="p-4 text-sm text-amber-400 bg-neutral-900 border border-neutral-800 rounded">{loadError}</div>
    {:else if tabs.length === 0}
      <div class="h-full flex flex-col items-center justify-center gap-3 text-center border border-dashed border-neutral-800 rounded-lg">
        <p class="text-neutral-300 font-medium">Belum ada sesi terminal yang dibuka.</p>
        <p class="text-neutral-500 text-sm max-w-sm">Pilih host dari halaman Hosts (atau Launch All dari sebuah Group) untuk membuka sesi SSH di sini.</p>
        <a href="/" class="px-4 py-2 bg-sky-600 hover:bg-sky-500 text-white text-sm font-medium rounded-md transition-colors">Ke halaman Hosts</a>
      </div>
    {:else}
      <div class="flex h-full gap-2 overflow-hidden">
        <!-- Terminal Panes -->
        <div class="flex-1 min-w-0 h-full overflow-hidden">
          {#if layout === 1 || tabs.length === 1}
            {@const h = pane(0)}
            {#if h}<TerminalPane host={h} onSplitRight={tabs.length > 1 ? () => layout = 3 : undefined} onSplitDown={tabs.length > 1 ? () => layout = 2 : undefined} onClose={() => close(h)} />{/if}
          {:else if layout === 2}
            <div class="flex flex-col h-full gap-2">
              {#each tabs as tab (tab.id)}
                <div class="flex-1 min-h-0"><TerminalPane host={tab.host} onClose={() => close(tab.host)} /></div>
              {/each}
            </div>
          {:else if layout === 3}
            <div class="flex h-full gap-2">
              {#each tabs as tab (tab.id)}
                <div class="flex-1 min-w-0"><TerminalPane host={tab.host} onClose={() => close(tab.host)} /></div>
              {/each}
            </div>
          {:else if layout === 4}
            <div class="grid grid-cols-2 grid-rows-2 h-full gap-2">
              {#each tabs.slice(0, 4) as tab (tab.id)}
                <div class="min-h-0 min-w-0"><TerminalPane host={tab.host} onClose={() => close(tab.host)} /></div>
              {/each}
            </div>
          {/if}
        </div>

        <!-- Collapsible Remote File Explorer Panel -->
        {#if showFiles && activeHost}
          <div class="w-80 lg:w-96 shrink-0 h-full overflow-hidden">
            <SessionFileManager
              host={activeHost}
              availableHosts={tabs.map(t => t.host)}
              onSelectHost={(h) => selectedHostId = h.id}
              onClose={() => showFiles = false}
            />
          </div>
        {/if}
      </div>
    {/if}
  </div>
</div>
