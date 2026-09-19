<script lang="ts">
  import { onMount } from 'svelte';
  import { listTunnels, saveTunnel, deleteTunnel, startTunnel, stopTunnel, type TunnelRecord, type ForwardType } from '$lib/api/tunnels';
  import { listHosts, type HostRecord } from '$lib/api/hosts';

  let tunnels: TunnelRecord[] = [];
  let hosts: HostRecord[] = [];
  let isLoading = true;
  let errorMsg = '';
  let successMsg = '';

  // Add modal
  let showAddModal = false;
  let formHostId = '';
  let formType: ForwardType = 'local';
  let formBindAddr = '127.0.0.1';
  let formBindPort = 8080;
  let formTargetAddr = '127.0.0.1';
  let formTargetPort = 80;
  let isSaving = false;

  async function loadData() {
    isLoading = true;
    errorMsg = '';
    try {
      [tunnels, hosts] = await Promise.all([listTunnels(), listHosts()]);
    } catch (e: any) {
      errorMsg = String(e);
    } finally {
      isLoading = false;
    }
  }

  onMount(loadData);

  function openAddModal() {
    formHostId = hosts.length > 0 ? hosts[0].id : '';
    formType = 'local';
    formBindAddr = '127.0.0.1';
    formBindPort = 8080;
    formTargetAddr = '127.0.0.1';
    formTargetPort = 80;
    showAddModal = true;
    errorMsg = '';
    successMsg = '';
  }

  async function handleSave() {
    if (!formHostId || !formBindPort || !formTargetPort) return;
    isSaving = true;
    errorMsg = '';
    try {
      await saveTunnel({
        hostId: formHostId,
        forwardType: formType,
        bindAddr: formBindAddr,
        bindPort: Number(formBindPort),
        targetAddr: formTargetAddr,
        targetPort: Number(formTargetPort),
      });
      showAddModal = false;
      await loadData();
    } catch (e: any) {
      errorMsg = String(e);
    } finally {
      isSaving = false;
    }
  }

  async function handleToggle(tunnel: TunnelRecord) {
    errorMsg = '';
    try {
      if (tunnel.isActive) {
        await stopTunnel(tunnel.id);
      } else {
        await startTunnel(tunnel.id);
      }
      await loadData();
    } catch (e: any) {
      errorMsg = String(e);
    }
  }

  async function handleDelete(id: string) {
    if (!confirm('Hapus konfigurasi port forwarding ini?')) return;
    errorMsg = '';
    try {
      await deleteTunnel(id);
      await loadData();
    } catch (e: any) {
      errorMsg = String(e);
    }
  }
</script>

<div class="max-w-4xl mx-auto space-y-6">
  <div class="flex items-center gap-3">
    <div class="p-2 bg-amber-500/10 text-amber-400 rounded-lg border border-amber-500/20">
      <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4"></path></svg>
    </div>
    <div class="flex-1">
      <h1 class="text-2xl font-bold text-white">Port Forwarding</h1>
      <p class="text-sm text-neutral-400">Manage Local, Remote, and Dynamic SSH tunnels.</p>
    </div>
    <button on:click={openAddModal} class="px-3 py-1.5 bg-sky-600 hover:bg-sky-500 text-white rounded-lg text-sm transition-colors shadow shadow-sky-600/20">
      Add Tunnel
    </button>
  </div>

  {#if errorMsg}
    <div class="p-4 bg-red-500/10 border border-red-500/20 rounded-lg text-red-400 text-sm">
      {errorMsg}
    </div>
  {/if}

  {#if isLoading}
    <div class="p-8 text-center text-neutral-500 text-sm">Loading tunnels...</div>
  {:else if tunnels.length === 0}
    <div class="p-8 border border-neutral-800 rounded-xl bg-neutral-900/30 text-center flex flex-col items-center justify-center">
      <svg class="w-12 h-12 text-neutral-600 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"></path></svg>
      <h3 class="text-lg font-medium text-white mb-2">No active tunnels</h3>
      <p class="text-sm text-neutral-400 max-w-md">Create port forwarding rules to securely tunnel local ports through your connected SSH hosts.</p>
      <button on:click={openAddModal} class="mt-6 px-4 py-2 bg-sky-600 hover:bg-sky-500 text-white rounded-lg text-sm font-medium transition-colors shadow-lg shadow-sky-600/20">
        Add Port Forwarding Rule
      </button>
    </div>
  {:else}
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      {#each tunnels as tun (tun.id)}
        {@const host = hosts.find(h => h.id === tun.hostId)}
        <div class="flex flex-col p-4 bg-neutral-900 border border-neutral-800 rounded-xl hover:border-neutral-700 transition-colors">
          <div class="flex items-center justify-between mb-2">
            <span class="font-medium text-white text-sm flex items-center gap-2">
              <span class="w-2 h-2 rounded-full {tun.isActive ? 'bg-emerald-400 animate-pulse' : 'bg-neutral-600'}"></span>
              {tun.bindAddr}:{tun.bindPort} &rarr; {tun.targetAddr}:{tun.targetPort}
            </span>
            <span class="text-xs px-2 py-0.5 rounded-full bg-neutral-800 text-neutral-400 font-mono capitalize">
              {tun.forwardType}
            </span>
          </div>

          <p class="text-xs text-neutral-400 mb-4">
            Via host: <span class="text-white font-medium">{host?.label || tun.hostId}</span>
          </p>

          <div class="mt-auto pt-3 border-t border-neutral-800/50 flex justify-between items-center">
            <button
              on:click={() => handleToggle(tun)}
              class="text-xs px-3 py-1 rounded font-medium transition-colors {tun.isActive ? 'bg-amber-500/10 text-amber-400 hover:bg-amber-500/20' : 'bg-emerald-500/10 text-emerald-400 hover:bg-emerald-500/20'}"
            >
              {tun.isActive ? 'Stop Tunnel' : 'Start Tunnel'}
            </button>
            <button
              on:click={() => handleDelete(tun.id)}
              class="text-xs text-rose-500 hover:text-rose-400 font-medium px-2 py-1 rounded hover:bg-rose-500/10 transition-colors"
            >
              Delete
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

{#if showAddModal}
<div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm">
  <div class="bg-neutral-900 border border-neutral-800 rounded-xl w-full max-w-md shadow-2xl p-6">
    <h3 class="text-lg font-semibold text-white mb-4">Add Port Forwarding Rule</h3>
    
    <div class="space-y-4">
      <div>
        <label class="block text-xs font-medium text-neutral-400 mb-1">Target Host</label>
        {#if hosts.length === 0}
          <div class="p-3 bg-neutral-950 border border-neutral-800 rounded-lg text-sm text-neutral-500">
            No hosts available. Add a host in Hosts tab first.
          </div>
        {:else}
          <select bind:value={formHostId} class="w-full bg-black/40 border border-neutral-800 rounded-lg px-3 py-2 text-sm text-white focus:outline-none focus:border-sky-500 transition-colors">
            {#each hosts as h}
              <option value={h.id}>{h.label} ({h.username}@{h.address})</option>
            {/each}
          </select>
        {/if}
      </div>

      <div>
        <label class="block text-xs font-medium text-neutral-400 mb-1">Forwarding Type</label>
        <select bind:value={formType} class="w-full bg-black/40 border border-neutral-800 rounded-lg px-3 py-2 text-sm text-white focus:outline-none focus:border-sky-500 transition-colors">
          <option value="local">Local (Forward local port to remote destination)</option>
          <option value="remote">Remote (Forward remote port to local)</option>
          <option value="dynamic">Dynamic (SOCKS5 Proxy)</option>
        </select>
      </div>

      <div class="grid grid-cols-2 gap-3">
        <div>
          <label class="block text-xs font-medium text-neutral-400 mb-1">Bind Address</label>
          <input type="text" bind:value={formBindAddr} class="w-full bg-black/40 border border-neutral-800 rounded-lg px-3 py-2 text-sm text-white focus:outline-none focus:border-sky-500 transition-colors" />
        </div>
        <div>
          <label class="block text-xs font-medium text-neutral-400 mb-1">Bind Port</label>
          <input type="number" bind:value={formBindPort} class="w-full bg-black/40 border border-neutral-800 rounded-lg px-3 py-2 text-sm text-white focus:outline-none focus:border-sky-500 transition-colors" />
        </div>
      </div>

      <div class="grid grid-cols-2 gap-3">
        <div>
          <label class="block text-xs font-medium text-neutral-400 mb-1">Target Address</label>
          <input type="text" bind:value={formTargetAddr} class="w-full bg-black/40 border border-neutral-800 rounded-lg px-3 py-2 text-sm text-white focus:outline-none focus:border-sky-500 transition-colors" />
        </div>
        <div>
          <label class="block text-xs font-medium text-neutral-400 mb-1">Target Port</label>
          <input type="number" bind:value={formTargetPort} class="w-full bg-black/40 border border-neutral-800 rounded-lg px-3 py-2 text-sm text-white focus:outline-none focus:border-sky-500 transition-colors" />
        </div>
      </div>
    </div>

    <div class="flex justify-end gap-3 mt-6">
      <button on:click={() => showAddModal = false} class="px-4 py-2 text-neutral-400 hover:text-white transition-colors text-sm font-medium">Cancel</button>
      <button on:click={handleSave} disabled={!formHostId || isSaving} class="px-4 py-2 bg-sky-600 hover:bg-sky-500 disabled:opacity-50 text-white rounded-lg text-sm font-medium transition-colors">
        {isSaving ? 'Saving...' : 'Save Rule'}
      </button>
    </div>
  </div>
</div>
{/if}