<script lang="ts">
  import { onMount } from 'svelte';
  import TerminalPane from '$lib/components/TerminalPane.svelte';
  import { listHosts, type HostRecord } from '$lib/api/hosts';

  // 1 = single, 2 = split horizontal, 3 = split vertical, 4 = grid 2x2
  let layout = $state(1);

  // Shown until real hosts are loaded from the Rust backend (or when running
  // in a plain browser during `vite dev`, where no Tauri backend exists).
  const fallbackHosts: HostRecord[] = [
    { id: 'fallback-1', label: 'Production VPS', address: '100.76.150.46', port: 22, username: 'root', authMethod: { type: 'password' }, tags: [], createdAt: 0, updatedAt: 0 },
    { id: 'fallback-2', label: 'App Node 1', address: '10.0.0.10', port: 22, username: 'root', authMethod: { type: 'password' }, tags: [], createdAt: 0, updatedAt: 0 },
    { id: 'fallback-3', label: 'App Node 2', address: '10.0.0.11', port: 22, username: 'root', authMethod: { type: 'password' }, tags: [], createdAt: 0, updatedAt: 0 },
    { id: 'fallback-4', label: 'Database Master', address: '10.0.1.1', port: 22, username: 'root', authMethod: { type: 'password' }, tags: [], createdAt: 0, updatedAt: 0 }
  ];

  let hosts = $state<HostRecord[]>(fallbackHosts);

  onMount(async () => {
    try {
      const saved = await listHosts();
      if (saved.length > 0) hosts = saved;
    } catch {
      // Tauri backend unavailable (e.g. plain browser dev) — keep fallback hosts.
    }
  });

  const pane = (i: number) => hosts[i % hosts.length];
</script>

<div class="h-full flex flex-col space-y-2">
  <div class="flex justify-between items-center bg-neutral-900 border border-neutral-800 rounded px-4 py-2 shrink-0">
    <h1 class="text-sm font-bold text-white tracking-tight">Active Sessions</h1>
    <div class="flex gap-1 bg-neutral-950 p-1 rounded border border-neutral-800">
      <button onclick={() => layout = 1} class="p-1 rounded {layout === 1 ? 'bg-sky-600/20 text-sky-400' : 'text-neutral-500 hover:text-neutral-300'}" title="Single">
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><rect x="3" y="3" width="18" height="18" rx="2" stroke-width="2"></rect></svg>
      </button>
      <button onclick={() => layout = 2} class="p-1 rounded {layout === 2 ? 'bg-sky-600/20 text-sky-400' : 'text-neutral-500 hover:text-neutral-300'}" title="Split Horizontal">
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-width="2" d="M3 12h18M5 3h14a2 2 0 012 2v14a2 2 0 01-2 2H5a2 2 0 01-2-2V5a2 2 0 012-2z"></path></svg>
      </button>
      <button onclick={() => layout = 3} class="p-1 rounded {layout === 3 ? 'bg-sky-600/20 text-sky-400' : 'text-neutral-500 hover:text-neutral-300'}" title="Split Vertical">
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-width="2" d="M12 3v18M5 3h14a2 2 0 012 2v14a2 2 0 01-2 2H5a2 2 0 01-2-2V5a2 2 0 012-2z"></path></svg>
      </button>
      <button onclick={() => layout = 4} class="p-1 rounded {layout === 4 ? 'bg-sky-600/20 text-sky-400' : 'text-neutral-500 hover:text-neutral-300'}" title="Grid 2x2">
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-width="2" d="M12 3v18M3 12h18M5 3h14a2 2 0 012 2v14a2 2 0 01-2 2H5a2 2 0 01-2-2V5a2 2 0 012-2z"></path></svg>
      </button>
    </div>
  </div>

  <div class="flex-1 w-full h-full overflow-hidden">
    {#if layout === 1}
      <TerminalPane host={pane(0)} onSplitRight={() => layout = 3} onSplitDown={() => layout = 2} />
    {:else if layout === 2}
      <div class="flex flex-col h-full gap-2">
        <div class="flex-1 min-h-0"><TerminalPane host={pane(0)} onSplitRight={() => layout = 4} onClose={() => layout = 1} /></div>
        <div class="flex-1 min-h-0"><TerminalPane host={pane(1)} onClose={() => layout = 1} /></div>
      </div>
    {:else if layout === 3}
      <div class="flex h-full gap-2">
        <div class="flex-1 min-w-0"><TerminalPane host={pane(0)} onSplitDown={() => layout = 4} onClose={() => layout = 1} /></div>
        <div class="flex-1 min-w-0"><TerminalPane host={pane(1)} onClose={() => layout = 1} /></div>
      </div>
    {:else if layout === 4}
      <div class="grid grid-cols-2 grid-rows-2 h-full gap-2">
        <div class="min-h-0 min-w-0"><TerminalPane host={pane(0)} onClose={() => layout = 3} /></div>
        <div class="min-h-0 min-w-0"><TerminalPane host={pane(1)} onClose={() => layout = 3} /></div>
        <div class="min-h-0 min-w-0"><TerminalPane host={pane(2)} onClose={() => layout = 2} /></div>
        <div class="min-h-0 min-w-0"><TerminalPane host={pane(3)} onClose={() => layout = 2} /></div>
      </div>
    {/if}
  </div>
</div>
