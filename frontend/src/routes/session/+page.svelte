<script lang="ts">
  import { page } from '$app/state';
  import TerminalPane from '$lib/components/TerminalPane.svelte';
  import SessionFileManager from '$lib/components/SessionFileManager.svelte';
  import WorkspaceMenu from '$lib/components/WorkspaceMenu.svelte';
  import { listHosts, type HostRecord } from '$lib/api/hosts';
  import { getTabs, openTab, closeTab } from '$lib/stores/sessionTabs.svelte';
  import { showToast } from '$lib/stores/uiNotifications.svelte';
  import type { Workspace } from '$lib/stores/workspaceStore.svelte';

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

  $effect(() => {
    if (tabs.length > 0 && (!selectedHostId || !tabs.some(t => t.host.id === selectedHostId))) {
      selectedHostId = tabs[0].host.id;
    }
  });

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

  async function handleLoadWorkspace(ws: Workspace) {
    try {
      const allHosts = await listHosts();
      for (const id of ws.hostIds) {
        const h = allHosts.find((item) => item.id === id);
        if (h) {
          openTab(h);
        }
      }
      if (ws.layout && [1, 2, 3, 4].includes(ws.layout)) {
        layout = ws.layout;
      }
      if (typeof ws.showFiles === 'boolean') {
        showFiles = ws.showFiles;
      }
      if (ws.hostIds.length > 0) {
        selectedHostId = ws.hostIds[0];
      }
      showToast(`Workspace "${ws.name}" loaded successfully`, 'success');
    } catch {
      showToast('Gagal memuat host untuk workspace', 'error');
    }
  }
</script>

<div class="h-full w-full flex flex-col bg-neutral-950 overflow-hidden">
  <!-- Compact Session Header Bar -->
  <div class="h-10 bg-neutral-900/90 border-b border-neutral-800 px-3 flex items-center justify-between shrink-0 select-none backdrop-blur-xs">
    <!-- Left: Active Host Info & Files Toggle -->
    <div class="flex items-center gap-2.5 min-w-0">
      <div class="flex items-center gap-2 min-w-0">
        <span class="text-xs font-bold text-white tracking-tight uppercase">Session</span>
        {#if activeHost}
          <span class="text-neutral-600 hidden sm:inline">•</span>
          <div class="flex items-center gap-1.5 min-w-0">
            <span class="w-2 h-2 rounded-full bg-emerald-500 shrink-0"></span>
            <span class="text-xs font-medium text-neutral-200 truncate max-w-[140px] sm:max-w-[200px]" title="{activeHost.label} ({activeHost.address})">
              {activeHost.label}
            </span>
            <span class="text-[11px] text-neutral-500 font-mono hidden md:inline truncate max-w-[130px]">
              ({activeHost.address})
            </span>
          </div>
        {/if}
        {#if tabs.length > 1}
          <span class="px-1.5 py-0.2 rounded text-[10px] font-mono bg-neutral-800 text-neutral-400 hidden sm:inline">
            {tabs.length} hosts
          </span>
        {/if}
      </div>

      {#if tabs.length > 0}
        <button
          onclick={() => showFiles = !showFiles}
          class="px-2 py-0.5 rounded text-xs font-medium border transition-colors flex items-center gap-1.5 {showFiles ? 'bg-sky-600/20 text-sky-400 border-sky-500/30 hover:bg-sky-600/30' : 'bg-neutral-950 text-neutral-400 border-neutral-800 hover:text-white hover:border-neutral-700'}"
          title={showFiles ? 'Hide Remote Files (SFTP)' : 'Show Remote Files (SFTP)'}
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"></path>
          </svg>
          <span class="hidden sm:inline">Files</span>
        </button>
      {/if}
    </div>

    <!-- Right: Split Controls (Only if multi-host) & Workspace Menu -->
    <div class="flex items-center gap-2 shrink-0">
      <!-- Split Controls: ONLY shown when tabs.length > 1 -->
      {#if tabs.length > 1}
        <div class="hidden sm:flex items-center gap-0.5 bg-neutral-950 p-0.5 rounded border border-neutral-800">
          <button
            onclick={() => layout = 1}
            class="p-1 rounded transition-colors {layout === 1 ? 'bg-sky-600/25 text-sky-400 border border-sky-500/40 shadow-xs' : 'text-neutral-400 hover:text-neutral-200 hover:bg-neutral-800/60 border border-transparent'}"
            title="Single View (1 pane)"
            aria-label="Single View"
          >
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><rect x="3" y="3" width="18" height="18" rx="2" stroke-width="2"></rect></svg>
          </button>
          <button
            onclick={() => layout = 2}
            disabled={tabs.length < 2}
            class="p-1 rounded transition-colors disabled:opacity-30 {layout === 2 ? 'bg-sky-600/25 text-sky-400 border border-sky-500/40 shadow-xs' : 'text-neutral-400 hover:text-neutral-200 hover:bg-neutral-800/60 border border-transparent'}"
            title="Split Horizontal (Top / Bottom)"
            aria-label="Split Horizontal"
          >
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-width="2" d="M3 12h18M5 3h14a2 2 0 012 2v14a2 2 0 01-2 2H5a2 2 0 01-2-2V5a2 2 0 012-2z"></path></svg>
          </button>
          <button
            onclick={() => layout = 3}
            disabled={tabs.length < 2}
            class="p-1 rounded transition-colors disabled:opacity-30 {layout === 3 ? 'bg-sky-600/25 text-sky-400 border border-sky-500/40 shadow-xs' : 'text-neutral-400 hover:text-neutral-200 hover:bg-neutral-800/60 border border-transparent'}"
            title="Split Vertical (Side by Side)"
            aria-label="Split Vertical"
          >
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-width="2" d="M12 3v18M5 3h14a2 2 0 012 2v14a2 2 0 01-2 2H5a2 2 0 01-2-2V5a2 2 0 012-2z"></path></svg>
          </button>
          <button
            onclick={() => layout = 4}
            disabled={tabs.length < 3}
            class="p-1 rounded transition-colors disabled:opacity-30 {layout === 4 ? 'bg-sky-600/25 text-sky-400 border border-sky-500/40 shadow-xs' : 'text-neutral-400 hover:text-neutral-200 hover:bg-neutral-800/60 border border-transparent'}"
            title="Grid 2x2 (4 Panes)"
            aria-label="Grid 2x2"
          >
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-width="2" d="M12 3v18M3 12h18M5 3h14a2 2 0 012 2v14a2 2 0 01-2 2H5a2 2 0 01-2-2V5a2 2 0 012-2z"></path></svg>
          </button>
        </div>
      {/if}

      <!-- Workspace Menu -->
      <WorkspaceMenu
        hostIds={tabs.map(t => t.host.id)}
        layout={layout}
        showFiles={showFiles}
        onLoad={handleLoadWorkspace}
      />
    </div>
  </div>

  <!-- Terminal & Content Viewport Area -->
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
                  onclick={() => selectedHostId = tab.host.id}
                  class="px-2.5 py-1 rounded text-xs font-mono transition-colors shrink-0 flex items-center gap-1.5 {selectedHostId === tab.host.id ? 'bg-sky-600 text-white font-semibold shadow-xs' : 'bg-neutral-900 text-neutral-400 hover:text-neutral-200 border border-neutral-800'}"
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
            {#if layout === 1 || tabs.length === 1}
              {@const h = activeHost || pane(0)}
              {#if h}
                <TerminalPane
                  host={h}
                  onSplitRight={tabs.length > 1 ? () => layout = 3 : undefined}
                  onSplitDown={tabs.length > 1 ? () => layout = 2 : undefined}
                  onClose={() => close(h)}
                />
              {/if}
            {:else if layout === 2}
              <div class="flex flex-col h-full gap-1 sm:gap-1.5">
                {#each tabs as tab (tab.id)}
                  <div class="flex-1 min-h-0">
                    <TerminalPane host={tab.host} onClose={() => close(tab.host)} />
                  </div>
                {/each}
              </div>
            {:else if layout === 3}
              <div class="flex h-full gap-1 sm:gap-1.5">
                {#each tabs as tab (tab.id)}
                  <div class="flex-1 min-w-0">
                    <TerminalPane host={tab.host} onClose={() => close(tab.host)} />
                  </div>
                {/each}
              </div>
            {:else if layout === 4}
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
        {#if showFiles && activeHost}
          <!-- Desktop: inline side panel -->
          <div class="hidden md:block w-80 lg:w-96 shrink-0 h-full overflow-hidden">
            <SessionFileManager
              host={activeHost}
              availableHosts={tabs.map(t => t.host)}
              onSelectHost={(h) => selectedHostId = h.id}
              onClose={() => showFiles = false}
            />
          </div>

          <!-- Mobile: full overlay drawer modal with close button -->
          <div class="md:hidden fixed inset-0 z-50 bg-black/70 backdrop-blur-xs flex flex-col justify-end p-2">
            <div class="w-full h-full max-h-[94vh] flex flex-col bg-neutral-950 rounded-lg shadow-2xl border border-neutral-800 overflow-hidden">
              <SessionFileManager
                host={activeHost}
                availableHosts={tabs.map(t => t.host)}
                onSelectHost={(h) => selectedHostId = h.id}
                onClose={() => showFiles = false}
              />
            </div>
          </div>
        {/if}
      </div>
    {/if}
  </div>
</div>
