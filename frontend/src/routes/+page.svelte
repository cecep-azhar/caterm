<script lang="ts">
  import { onMount } from 'svelte';
  import { listHosts, saveHost, deleteHost, type HostRecord, type HostInput } from '$lib/api/hosts';
  import { listKeys, type KeyRecord } from '$lib/api/keys';

  let hosts = $state<HostRecord[]>([]);
  let vaultKeys = $state<KeyRecord[]>([]);
  let searchQuery = $state('');
  let isAddModalOpen = $state(false);
  let isLoading = $state(false);
  let errorMsg = $state('');
  let editingId = $state<string | null>(null);
  let editingHadSecret = $state(false);

  // Form State
  let formLabel = $state('');
  let formAddress = $state('');
  let formPort = $state(22);
  let formUsername = $state('root');
  let formAuthType = $state<'password' | 'key' | 'keyId'>('password');
  let formKeyPath = $state('');
  let formKeyId = $state('');
  let formSecret = $state('');
  let formTags = $state('');

  onMount(async () => {
    await refreshHosts();
  });

  async function refreshHosts() {
    try {
      hosts = await listHosts();
      vaultKeys = await listKeys();
    } catch (e: any) {
      errorMsg = String(e);
    }
  }

  function openAddModal() {
    editingId = null;
    editingHadSecret = false;
    formLabel = '';
    formAddress = '';
    formPort = 22;
    formUsername = 'root';
    formAuthType = 'password';
    formKeyPath = '';
    formSecret = '';
    formTags = '';
    errorMsg = '';
    isAddModalOpen = true;
  }

  function openEditModal(host: HostRecord) {
    editingId = host.id;
    editingHadSecret = host.hasSecret;
    formLabel = host.label;
    formAddress = host.address;
    formPort = host.port;
    formUsername = host.username;
    formAuthType = host.authMethod.type;
    formKeyPath = host.authMethod.type === 'key' ? host.authMethod.path : '';
    formKeyId = host.authMethod.type === 'keyId' ? host.authMethod.id : '';
    formSecret = '';
    formTags = host.tags.join(', ');
    errorMsg = '';
    isAddModalOpen = true;
  }

  async function handleSaveHost() {
    if (!formLabel || !formAddress || !formUsername) {
      errorMsg = 'Label, Host/IP, and Username are required.';
      return;
    }
    isLoading = true;
    errorMsg = '';
    try {
      const input: HostInput = {
        id: editingId ?? undefined,
        label: formLabel,
        address: formAddress,
        port: Number(formPort) || 22,
        username: formUsername,
        authMethod: formAuthType === 'password' ? { type: 'password' } : (formAuthType === 'keyId' ? { type: 'keyId', id: formKeyId } : { type: 'key', path: formKeyPath }),
        tags: formTags.split(',').map(t => t.trim()).filter(Boolean),
        // Blank = leave whatever's stored untouched (edit) or no secret at all (create).
        secret: formSecret ? formSecret : undefined
      };
      await saveHost(input);
      isAddModalOpen = false;
      await refreshHosts();
    } catch (err: any) {
      errorMsg = typeof err === 'string' ? err : (err?.message || 'Failed to save host');
    } finally {
      isLoading = false;
    }
  }

  async function handleDelete(id: string) {
    if (confirm('Are you sure you want to delete this host?')) {
      try {
        await deleteHost(id);
        await refreshHosts();
      } catch (err: any) {
        alert(err?.message || 'Failed to delete host');
      }
    }
  }

  let filteredHosts = $derived(
    hosts.filter(h => 
      h.label.toLowerCase().includes(searchQuery.toLowerCase()) ||
      h.address.toLowerCase().includes(searchQuery.toLowerCase()) ||
      h.username.toLowerCase().includes(searchQuery.toLowerCase())
    )
  );
</script>

<div class="max-w-6xl mx-auto space-y-6">
  <!-- Header & Actions -->
  <div class="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
    <div>
      <h1 class="text-2xl font-bold text-white tracking-tight">Hosts Management</h1>
      <p class="text-sm text-neutral-400 mt-1">Manage saved SSH hosts, connection profiles, and credentials.</p>
    </div>
    
    <div class="flex items-center gap-3 w-full sm:w-auto">
      <div class="relative flex-1 sm:w-64">
        <svg class="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-neutral-500" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"></path></svg>
        <input 
          type="text"
          bind:value={searchQuery}
          placeholder="Search hosts..."
          class="w-full pl-9 pr-4 py-2 bg-neutral-900 border border-neutral-800 rounded-lg text-sm text-white placeholder-neutral-500 focus:outline-none focus:border-sky-500"
        />
      </div>

      <button 
        onclick={openAddModal}
        class="px-4 py-2 bg-sky-600 hover:bg-sky-500 text-white text-sm font-medium rounded-lg shadow-lg shadow-sky-600/20 transition-colors flex items-center gap-2 shrink-0">
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"></path></svg>
        Add Host
      </button>
    </div>
  </div>

  <!-- Hosts Table / Cards -->
  {#if filteredHosts.length === 0}
    <div class="p-12 border border-neutral-800 rounded-xl bg-neutral-900/30 text-center flex flex-col items-center justify-center">
      <div class="w-12 h-12 rounded-full bg-neutral-800 flex items-center justify-center text-neutral-500 mb-4">
        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M12 5l7 7-7 7"></path></svg>
      </div>
      <h3 class="text-lg font-medium text-white mb-1">No saved hosts found</h3>
      <p class="text-sm text-neutral-400 max-w-sm mb-6">Create your first host entry to connect via interactive SSH PTY terminal.</p>
      <button 
        onclick={openAddModal}
        class="px-4 py-2 bg-sky-600 hover:bg-sky-500 text-white text-sm font-medium rounded-lg transition-colors">
        + Add Host
      </button>
    </div>
  {:else}
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      {#each filteredHosts as host (host.id)}
        <div class="p-5 bg-neutral-900 border border-neutral-800 rounded-xl hover:border-neutral-700 transition-colors flex flex-col justify-between group">
          <div>
            <div class="flex items-center justify-between mb-2">
              <span class="font-bold text-white text-base truncate">{host.label}</span>
              <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                <button
                  onclick={() => openEditModal(host)}
                  class="p-1 text-neutral-400 hover:text-sky-400 rounded transition-colors"
                  title="Edit Host">
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"></path></svg>
                </button>
                <button
                  onclick={() => handleDelete(host.id)}
                  class="p-1 text-neutral-400 hover:text-rose-400 rounded transition-colors"
                  title="Delete Host">
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"></path></svg>
                </button>
              </div>
            </div>

            <div class="space-y-1 font-mono text-xs text-neutral-400">
              <p><span class="text-neutral-500">Address:</span> <span class="text-sky-400">{host.username}@{host.address}:{host.port}</span></p>
              <p><span class="text-neutral-500">Auth:</span> <span class="text-neutral-300">{host.authMethod.type}</span> {#if host.hasSecret}<span class="text-emerald-500">• tersimpan</span>{:else}<span class="text-amber-500">• belum ada password</span>{/if}</p>
            </div>

            {#if host.tags.length > 0}
              <div class="flex flex-wrap gap-1 mt-3">
                {#each host.tags as tag}
                  <span class="px-2 py-0.5 rounded text-[10px] bg-neutral-800 text-neutral-300 border border-neutral-700">{tag}</span>
                {/each}
              </div>
            {/if}
          </div>

          <div class="pt-4 mt-4 border-t border-neutral-800/80 flex items-center justify-between">
            <span class="text-[10px] text-neutral-500">ID: {host.id.slice(0, 8)}</span>
            <a
              href="/session?host={host.id}"
              class="px-3 py-1.5 bg-sky-600/20 hover:bg-sky-600 text-sky-400 hover:text-white rounded-md text-xs font-medium border border-sky-500/30 transition-all flex items-center gap-1.5">
              <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14 5l7 7m0 0l-7 7m7-7H3"></path></svg>
              Connect
            </a>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<!-- Add Host Modal -->
{#if isAddModalOpen}
  <div class="fixed inset-0 bg-black/80 flex items-center justify-center p-4 z-50">
    <div class="bg-neutral-900 border border-neutral-800 rounded-xl p-6 max-w-lg w-full space-y-4 shadow-2xl">
      <div class="flex items-center justify-between border-b border-neutral-800 pb-3">
        <h3 class="text-lg font-bold text-white">{editingId ? 'Edit SSH Host' : 'Add New SSH Host'}</h3>
        <button onclick={() => isAddModalOpen = false} class="text-neutral-500 hover:text-neutral-300 text-xl font-bold">×</button>
      </div>

      {#if errorMsg}
        <div class="p-3 text-xs rounded bg-rose-500/10 border border-rose-500/30 text-rose-400">
          {errorMsg}
        </div>
      {/if}

      <div class="space-y-4 text-sm">
        <div>
          <label class="block text-xs font-medium text-neutral-400 mb-1">Host Label / Name *</label>
          <input 
            type="text" 
            bind:value={formLabel} 
            placeholder="Production VPS / YPC Server"
            class="w-full px-3 py-2 bg-neutral-950 border border-neutral-800 rounded-lg text-white focus:outline-none focus:border-sky-500"
          />
        </div>

        <div class="grid grid-cols-3 gap-3">
          <div class="col-span-2">
            <label class="block text-xs font-medium text-neutral-400 mb-1">Host / IP Address *</label>
            <input 
              type="text" 
              bind:value={formAddress} 
              placeholder="100.76.150.46 or vps.domain.com"
              class="w-full px-3 py-2 bg-neutral-950 border border-neutral-800 rounded-lg text-white focus:outline-none focus:border-sky-500"
            />
          </div>
          <div>
            <label class="block text-xs font-medium text-neutral-400 mb-1">Port</label>
            <input 
              type="number" 
              bind:value={formPort} 
              placeholder="22"
              class="w-full px-3 py-2 bg-neutral-950 border border-neutral-800 rounded-lg text-white focus:outline-none focus:border-sky-500"
            />
          </div>
        </div>

        <div>
          <label class="block text-xs font-medium text-neutral-400 mb-1">Username *</label>
          <input 
            type="text" 
            bind:value={formUsername} 
            placeholder="root / cecep"
            class="w-full px-3 py-2 bg-neutral-950 border border-neutral-800 rounded-lg text-white focus:outline-none focus:border-sky-500"
          />
        </div>

        <div>
          <label class="block text-xs font-medium text-neutral-400 mb-1">Authentication Method</label>
          <div class="flex gap-4 mb-2">
            <label class="flex items-center gap-2 text-xs text-neutral-300">
              <input type="radio" value="password" bind:group={formAuthType} class="text-sky-600 focus:ring-0" />
              Password
            </label>
            <label class="flex items-center gap-2 text-xs text-neutral-300">
              <input type="radio" value="keyId" bind:group={formAuthType} class="text-sky-600 focus:ring-0" />
              Vault Key
            </label>
            <label class="flex items-center gap-2 text-xs text-neutral-300">
              <input type="radio" value="key" bind:group={formAuthType} class="text-sky-600 focus:ring-0" />
              Local File
            </label>
          </div>
          {#if formAuthType === 'key'}
            <input
              type="text"
              bind:value={formKeyPath}
              placeholder="/home/user/.ssh/id_ed25519"
              class="w-full px-3 py-2 bg-neutral-950 border border-neutral-800 rounded-lg text-white focus:outline-none focus:border-sky-500 mb-3"
            />
          {:else if formAuthType === 'keyId'}
            <select
              bind:value={formKeyId}
              class="w-full px-3 py-2 bg-neutral-950 border border-neutral-800 rounded-lg text-white focus:outline-none focus:border-sky-500 mb-3"
            >
              <option value="" disabled>-- Select a key from vault --</option>
              {#each vaultKeys as k}
                <option value={k.id}>{k.name} ({k.algorithm})</option>
              {/each}
            </select>
          {/if}
          {#if formAuthType !== 'keyId'}
          <label class="block text-xs font-medium text-neutral-400 mb-1">
            {formAuthType === 'key' ? 'Key Passphrase (optional)' : 'Password'}
          </label>
          <input
            type="password"
            bind:value={formSecret}
            placeholder={editingId && editingHadSecret ? 'Kosongkan untuk tidak mengubah' : (formAuthType === 'key' ? 'Kosongkan jika key tidak berpassphrase' : 'Wajib diisi agar bisa connect')}
            class="w-full px-3 py-2 bg-neutral-950 border border-neutral-800 rounded-lg text-white focus:outline-none focus:border-sky-500"
          />
          {/if}
          <p class="text-neutral-500 text-xs mt-1">Disimpan terenkripsi (AES-256-GCM) di database lokal — tidak pernah dikirim balik ke UI.</p>
        </div>

        <div>
          <label class="block text-xs font-medium text-neutral-400 mb-1">Tags (Comma-separated)</label>
          <input 
            type="text" 
            bind:value={formTags} 
            placeholder="prod, vps, tailscale"
            class="w-full px-3 py-2 bg-neutral-950 border border-neutral-800 rounded-lg text-white focus:outline-none focus:border-sky-500"
          />
        </div>
      </div>

      <div class="flex justify-end gap-3 pt-4 border-t border-neutral-800">
        <button 
          onclick={() => isAddModalOpen = false} 
          class="px-4 py-2 bg-neutral-800 hover:bg-neutral-700 text-neutral-300 rounded-lg text-sm font-medium">
          Cancel
        </button>
        <button 
          onclick={handleSaveHost}
          disabled={isLoading}
          class="px-4 py-2 bg-sky-600 hover:bg-sky-500 disabled:opacity-50 text-white rounded-lg text-sm font-medium shadow-lg shadow-sky-600/20">
          {isLoading ? 'Saving...' : 'Save Host'}
        </button>
      </div>
    </div>
  </div>
{/if}
