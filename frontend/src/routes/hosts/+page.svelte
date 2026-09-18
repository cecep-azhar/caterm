<script lang="ts">
  import { onMount } from 'svelte';
  import { listHosts, saveHost, deleteHost, type HostRecord } from '$lib/api/hosts';

  let isAddModalOpen = $state(false);
  let backendAvailable = $state(true);
  let hosts = $state<HostRecord[]>([]);

  let newHost = $state({
    label: "",
    ip: "",
    port: 22,
    username: "root",
    authType: "password" as "password" | "key",
    keyPath: "",
    tags: ""
  });

  onMount(async () => {
    try {
      hosts = await listHosts();
    } catch {
      backendAvailable = false;
    }
  });

  async function addHost(e: Event) {
    e.preventDefault();
    if (!newHost.label || !newHost.ip) return;

    const input = {
      label: newHost.label,
      address: newHost.ip,
      port: newHost.port,
      username: newHost.username,
      authMethod:
        newHost.authType === 'key'
          ? ({ type: 'key', path: newHost.keyPath } as const)
          : ({ type: 'password' } as const),
      tags: newHost.tags.split(',').map((t) => t.trim()).filter(Boolean)
    };

    try {
      const saved = await saveHost(input);
      hosts = [...hosts, saved];
    } catch {
      backendAvailable = false;
    }

    newHost = { label: "", ip: "", port: 22, username: "root", authType: "password", keyPath: "", tags: "" };
    isAddModalOpen = false;
  }

  async function removeHost(id: string) {
    try {
      await deleteHost(id);
      hosts = hosts.filter((h) => h.id !== id);
    } catch {
      backendAvailable = false;
    }
  }
</script>

<div class="max-w-5xl mx-auto space-y-6">
  <div class="flex justify-between items-center">
    <div>
      <h1 class="text-2xl font-bold text-white tracking-tight">Hosts Management</h1>
      <p class="text-neutral-400 text-sm mt-1">Manage your remote servers and zero-knowledge local key associations.</p>
      {#if !backendAvailable}
        <p class="text-amber-500 text-xs mt-1">Tauri backend not detected — changes won't be saved to disk.</p>
      {/if}
    </div>
    <button
      onclick={() => isAddModalOpen = true}
      class="px-4 py-2 bg-sky-600 hover:bg-sky-500 text-white font-medium text-sm rounded-md transition-colors flex items-center gap-2">
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"></path></svg>
      Add Host
    </button>
  </div>

  <!-- Hosts Table / List -->
  <div class="bg-neutral-900 border border-neutral-800 rounded-lg overflow-hidden">
    <table class="w-full text-left text-sm text-neutral-300">
      <thead class="bg-neutral-950 text-neutral-400 text-xs uppercase border-b border-neutral-800">
        <tr>
          <th class="px-4 py-3">Label</th>
          <th class="px-4 py-3">Address</th>
          <th class="px-4 py-3">Username</th>
          <th class="px-4 py-3">Tags</th>
          <th class="px-4 py-3 text-right">Actions</th>
        </tr>
      </thead>
      <tbody class="divide-y divide-neutral-800">
        {#each hosts as host}
          <tr class="hover:bg-neutral-800/50 transition-colors">
            <td class="px-4 py-3 font-medium text-white">{host.label}</td>
            <td class="px-4 py-3 font-mono text-neutral-400">{host.address}:{host.port}</td>
            <td class="px-4 py-3 font-mono text-neutral-400">{host.username}</td>
            <td class="px-4 py-3">
              <div class="flex gap-1 flex-wrap">
                {#each host.tags as tag}
                  <span class="px-2 py-0.5 bg-neutral-800 border border-neutral-700 text-neutral-400 rounded text-xs">{tag}</span>
                {/each}
              </div>
            </td>
            <td class="px-4 py-3 text-right space-x-2">
              <button class="px-3 py-1 bg-neutral-800 hover:bg-sky-600 hover:text-white rounded text-xs transition-colors">Connect</button>
              <button onclick={() => removeHost(host.id)} class="px-3 py-1 bg-neutral-800 hover:bg-red-600 hover:text-white rounded text-xs transition-colors">Delete</button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</div>

<!-- Add Host Modal -->
{#if isAddModalOpen}
  <div class="fixed inset-0 bg-black/70 flex items-center justify-center p-4 z-50">
    <div class="bg-neutral-900 border border-neutral-800 rounded-lg p-6 max-w-lg w-full space-y-4">
      <h3 class="text-xl font-bold text-white">Add New Host</h3>

      <form onsubmit={addHost} class="space-y-4">
        <div>
          <label for="host-label" class="block text-xs font-medium text-neutral-400 uppercase mb-1">Label</label>
          <input id="host-label" bind:value={newHost.label} required placeholder="e.g. Production VPS" class="w-full px-3 py-2 bg-neutral-950 border border-neutral-800 rounded text-sm text-white focus:outline-none focus:border-sky-500" />
        </div>

        <div class="grid grid-cols-3 gap-3">
          <div class="col-span-2">
            <label for="host-ip" class="block text-xs font-medium text-neutral-400 uppercase mb-1">IP / Hostname</label>
            <input id="host-ip" bind:value={newHost.ip} required placeholder="192.168.1.1 or example.com" class="w-full px-3 py-2 bg-neutral-950 border border-neutral-800 rounded text-sm text-white focus:outline-none focus:border-sky-500" />
          </div>
          <div>
            <label for="host-port" class="block text-xs font-medium text-neutral-400 uppercase mb-1">Port</label>
            <input id="host-port" type="number" bind:value={newHost.port} required class="w-full px-3 py-2 bg-neutral-950 border border-neutral-800 rounded text-sm text-white focus:outline-none focus:border-sky-500" />
          </div>
        </div>

        <div>
          <label for="host-username" class="block text-xs font-medium text-neutral-400 uppercase mb-1">Username</label>
          <input id="host-username" bind:value={newHost.username} required class="w-full px-3 py-2 bg-neutral-950 border border-neutral-800 rounded text-sm text-white focus:outline-none focus:border-sky-500" />
        </div>

        <div>
          <label for="host-auth-type" class="block text-xs font-medium text-neutral-400 uppercase mb-1">Auth Method</label>
          <select id="host-auth-type" bind:value={newHost.authType} class="w-full px-3 py-2 bg-neutral-950 border border-neutral-800 rounded text-sm text-white focus:outline-none focus:border-sky-500">
            <option value="password">Password</option>
            <option value="key">SSH Key</option>
          </select>
          <p class="text-neutral-500 text-xs mt-1">Credentials aren't stored yet — that lands with the encrypted vault. Only a key file path is remembered for now.</p>
        </div>

        {#if newHost.authType === 'key'}
          <div>
            <label for="host-keypath" class="block text-xs font-medium text-neutral-400 uppercase mb-1">Private Key Path</label>
            <input id="host-keypath" bind:value={newHost.keyPath} placeholder="~/.ssh/id_ed25519" class="w-full px-3 py-2 bg-neutral-950 border border-neutral-800 rounded text-sm text-white focus:outline-none focus:border-sky-500" />
          </div>
        {/if}

        <div>
          <label for="host-tags" class="block text-xs font-medium text-neutral-400 uppercase mb-1">Tags (comma separated)</label>
          <input id="host-tags" bind:value={newHost.tags} placeholder="production, vps, web" class="w-full px-3 py-2 bg-neutral-950 border border-neutral-800 rounded text-sm text-white focus:outline-none focus:border-sky-500" />
        </div>

        <div class="flex justify-end gap-2 pt-2">
          <button
            type="button"
            onclick={() => isAddModalOpen = false}
            class="px-4 py-2 bg-neutral-800 hover:bg-neutral-700 text-neutral-300 rounded text-sm font-medium">
            Cancel
          </button>
          <button
            type="submit"
            class="px-4 py-2 bg-sky-600 hover:bg-sky-500 text-white rounded text-sm font-medium">
            Save Host
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}
