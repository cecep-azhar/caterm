<script lang="ts">
  import { onMount } from 'svelte';
  import { listHosts, saveHost, deleteHost, type HostRecord, type HostInput } from '$lib/api/hosts';
  import { listKeys, type KeyRecord } from '$lib/api/keys';
  import HostDetailPanel from '$lib/components/HostDetailPanel.svelte';
  import OsIcon from '$lib/components/OsIcon.svelte';
  import { openSession } from '$lib/nav';
  import { showToast, confirmModal } from '$lib/stores/uiNotifications.svelte';

  let hosts = $state<HostRecord[]>([]);
  let vaultKeys = $state<KeyRecord[]>([]);
  let searchQuery = $state('');
  let isAddModalOpen = $state(false);
  let isLoading = $state(false);
  let errorMsg = $state('');
  let editingId = $state<string | null>(null);
  let editingHadSecret = $state(false);

  // Bulk selection + detail slide-over. Both buttons on the host card were previously inert
  // markup with no onclick; this is the state behind them.
  let selectedIds = $state<string[]>([]);
  let detailHost = $state<HostRecord | null>(null);

  const isSelected = (id: string) => selectedIds.includes(id);

  function toggleSelected(id: string) {
    selectedIds = isSelected(id) ? selectedIds.filter((x) => x !== id) : [...selectedIds, id];
  }

  function clearSelection() {
    selectedIds = [];
  }

  /** Opens every selected host as tabs in one session, which is what a split view needs. */
  function connectSelected() {
    if (selectedIds.length === 0) return;
    const ids = [...selectedIds];
    clearSelection();
    void openSession(ids);
  }

  async function deleteSelected() {
    const count = selectedIds.length;
    if (count === 0) return;
    const ok = await confirmModal(
      `Delete ${count} selected host(s)? Stored credentials will also be permanently deleted and cannot be recovered.`,
      'Delete Selected Hosts',
      true,
      'Delete',
      'Cancel'
    );
    if (!ok) return;

    const failed: string[] = [];
    for (const id of selectedIds) {
      try {
        await deleteHost(id);
      } catch {
        failed.push(hosts.find((h) => h.id === id)?.label ?? id);
      }
    }
    clearSelection();
    await refreshHosts();

    if (failed.length > 0) {
      showToast(`Failed to delete: ${failed.join(', ')}`, 'error');
    } else {
      showToast(`${count} host(s) deleted.`, 'success');
    }
  }

  // Form State
  let formLabel = $state('');
  let formAddress = $state('');
  let formPort = $state(22);
  let formUsername = $state('root');
  let formAuthType = $state<'password' | 'key' | 'keyId'>('password');
  let formKeyPath = $state('');
  let formKeyId = $state('');
  let formSecret = $state('');
  let showFormSecret = $state(false);
  let formTags = $state('');
  let formOs = $state('');

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
    formOs = '';
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
    formOs = host.os || '';
    errorMsg = '';
    isAddModalOpen = true;
  }

  async function handleSaveHost() {
    if (!formLabel || !formAddress || !formUsername) {
      errorMsg = 'Label, Host/IP, and Username are required.';
      showToast(errorMsg, 'error');
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
        os: formOs ? formOs : undefined,
        // Blank = leave whatever's stored untouched (edit) or no secret at all (create).
        secret: formSecret ? formSecret : undefined
      };
      await saveHost(input);
      isAddModalOpen = false;
      showToast(editingId ? 'Host successfully updated' : 'Host successfully added', 'success');
      await refreshHosts();
    } catch (err: any) {
      errorMsg = typeof err === 'string' ? err : (err?.message || 'Failed to save host');
      showToast(errorMsg, 'error');
    } finally {
      isLoading = false;
    }
  }

  async function handleDelete(id: string) {
    const confirmed = await confirmModal('Are you sure you want to delete this host from the database?', 'Delete Host', true, 'Delete', 'Cancel');
    if (confirmed) {
      try {
        await deleteHost(id);
        showToast('Host successfully deleted', 'success');
        await refreshHosts();
      } catch (err: any) {
        showToast(err?.message || 'Failed to delete host', 'error');
      }
    }
  }

  async function handleClone(host: HostRecord) {
    try {
      const input: HostInput = {
        label: `${host.label} (Copy)`,
        address: host.address,
        port: host.port,
        username: host.username,
        authMethod: host.authMethod,
        tags: [...host.tags],
        os: host.os
      };
      await saveHost(input);
      showToast(`Host "${host.label}" cloned successfully`, 'success');
      await refreshHosts();
    } catch (err: any) {
      showToast(err?.message || 'Failed to clone host', 'error');
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
  <div class="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 pb-4 border-b border-neutral-200 dark:border-neutral-800/80 mb-6">
    <div>
      <h1 class="text-2xl font-bold text-neutral-900 dark:text-white tracking-tight">Hosts Management</h1>
      <p class="text-sm text-neutral-500 dark:text-neutral-400 mt-1">Manage saved SSH hosts, connection profiles, and credentials.</p>
    </div>
    
    <div class="flex items-center gap-3 w-full sm:w-auto">
      <div class="relative flex-1 sm:w-64">
        <svg class="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-neutral-400 dark:text-neutral-500" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"></path></svg>
        <input 
          type="text"
          bind:value={searchQuery}
          placeholder="Search hosts..."
          class="w-full pl-9 pr-4 py-2 bg-white dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 rounded-lg text-sm text-neutral-900 dark:text-white placeholder-neutral-400 dark:placeholder-neutral-500 focus:outline-none focus:border-sky-500 shadow-sm dark:shadow-none"
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

  <!-- Bulk actions: only present once at least one host is ticked, so the page stays quiet
       when you are not doing a multi-host operation. -->
  {#if selectedIds.length > 0}
    <div class="flex flex-wrap items-center gap-3 p-3 rounded-lg bg-sky-50 dark:bg-sky-950/40 border border-sky-200 dark:border-sky-900">
      <span class="text-sm font-medium text-sky-800 dark:text-sky-300">
        {selectedIds.length} host(s) selected
      </span>
      <div class="flex items-center gap-2 ml-auto">
        <button
          onclick={connectSelected}
          class="px-3 py-1.5 bg-sky-600 hover:bg-sky-500 text-white rounded-lg text-xs font-medium transition-colors"
        >
          Connect All
        </button>
        <button
          onclick={deleteSelected}
          class="px-3 py-1.5 rounded-lg text-xs font-medium border border-rose-300 dark:border-rose-900 text-rose-600 dark:text-rose-400 hover:bg-rose-50 dark:hover:bg-rose-950/40 transition-colors"
        >
          Delete
        </button>
        <button
          onclick={clearSelection}
          class="px-3 py-1.5 rounded-lg text-xs font-medium text-neutral-600 dark:text-neutral-400 hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors"
        >
          Deselect
        </button>
      </div>
    </div>
  {/if}

  <!-- Hosts Table / Cards -->
  {#if filteredHosts.length === 0}
    <div class="p-12 border border-neutral-200 dark:border-neutral-800 rounded-xl bg-white/70 dark:bg-neutral-900/30 text-center flex flex-col items-center justify-center shadow-sm dark:shadow-none">
      <div class="w-12 h-12 rounded-full bg-neutral-100 dark:bg-neutral-800 flex items-center justify-center text-neutral-400 dark:text-neutral-500 mb-4">
        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M12 5l7 7-7 7"></path></svg>
      </div>
      <h3 class="text-lg font-medium text-neutral-900 dark:text-white mb-1">No saved hosts found</h3>
      <p class="text-sm text-neutral-500 dark:text-neutral-400 max-w-sm mb-6">Create your first host entry to connect via interactive SSH PTY terminal.</p>
      <button 
        onclick={openAddModal}
        class="px-4 py-2 bg-sky-600 hover:bg-sky-500 text-white text-sm font-medium rounded-lg transition-colors">
        + Add Host
      </button>
    </div>
  {:else}
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      {#each filteredHosts as host (host.id)}
        <div class="p-5 bg-white dark:bg-neutral-900 border rounded-xl shadow-sm dark:shadow-none transition-colors flex flex-col justify-between group {isSelected(host.id) ? 'border-sky-500 dark:border-sky-500 ring-1 ring-sky-500/40' : 'border-neutral-200 dark:border-neutral-800 hover:border-neutral-300 dark:hover:border-neutral-700'}">
          <div>
            <div class="flex items-center justify-between mb-2">
              <div class="flex items-center gap-2 min-w-0">
                <OsIcon os={host.os} name={host.label} tags={host.tags} address={host.address} size={18} />
                <span class="font-bold text-neutral-900 dark:text-white text-base truncate">{host.label}</span>
              </div>
              <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                <button
                  onclick={() => openEditModal(host)}
                  class="p-1 text-neutral-400 hover:text-sky-500 dark:hover:text-sky-400 rounded transition-colors"
                  title="Edit Host">
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"></path></svg>
                </button>
                <button
                  onclick={() => handleDelete(host.id)}
                  class="p-1 text-neutral-400 hover:text-rose-500 dark:hover:text-rose-400 rounded transition-colors"
                  title="Delete Host">
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"></path></svg>
                </button>
              </div>
            </div>

            <div class="space-y-1 font-mono text-xs text-neutral-500 dark:text-neutral-400">
              <p><span class="text-neutral-400 dark:text-neutral-500">Address:</span> <span class="text-sky-600 dark:text-sky-400">{host.username}@{host.address}:{host.port}</span></p>
              <p><span class="text-neutral-400 dark:text-neutral-500">Auth:</span> <span class="text-neutral-700 dark:text-neutral-300">{host.authMethod.type}</span> {#if host.hasSecret}<span class="text-emerald-600 dark:text-emerald-500">• saved</span>{:else}<span class="text-amber-600 dark:text-amber-500">• no password</span>{/if}</p>
            </div>

            {#if host.tags.length > 0}
              <div class="flex flex-wrap gap-1 mt-3">
                {#each host.tags as tag}
                  <span class="px-2 py-0.5 rounded text-[10px] bg-neutral-100 dark:bg-neutral-800 text-neutral-700 dark:text-neutral-300 border border-neutral-200 dark:border-neutral-700">{tag}</span>
                {/each}
              </div>
            {/if}
          </div>

          <div class="pt-4 mt-4 border-t border-neutral-100 dark:border-neutral-800/80 flex items-center justify-start gap-2">
            <!-- 1. Connect: Lightning SVG, primary button -->
            <button
              onclick={() => openSession(host.id)}
              class="w-8 h-8 flex items-center justify-center bg-sky-600 hover:bg-sky-500 text-white rounded-lg text-xs font-semibold shadow-sm transition-all shrink-0"
              title="Connect"
              aria-label="Connect"
            >
              <svg class="w-4 h-4 fill-current" viewBox="0 0 24 24">
                <path d="M13 2L3 14h7v8l11-12h-8l1-8z" />
              </svg>
            </button>

            <!-- 2. Clone: Duplicate/Copy SVG -->
            <button
              onclick={() => handleClone(host)}
              class="w-8 h-8 flex items-center justify-center bg-neutral-100 hover:bg-neutral-200 dark:bg-neutral-800 dark:hover:bg-neutral-700 text-neutral-700 dark:text-neutral-300 rounded-lg text-xs font-semibold border border-neutral-300 dark:border-neutral-700 transition-all shrink-0 shadow-sm"
              title="Clone Host"
              aria-label="Clone Host"
            >
              <svg class="w-4 h-4 text-sky-600 dark:text-sky-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
              </svg>
            </button>

            <!-- 3. Check: Select checkbox SVG -->
            <button
              onclick={() => toggleSelected(host.id)}
              aria-pressed={isSelected(host.id)}
              class="w-8 h-8 flex items-center justify-center rounded-lg text-xs font-medium border transition-all shrink-0 {isSelected(host.id) ? 'bg-sky-600 border-sky-600 text-white' : 'text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white border-neutral-200 dark:border-neutral-700 hover:bg-neutral-100 dark:hover:bg-neutral-800'}"
              title="Select for batch action"
              aria-label="Select for batch action"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"></path></svg>
            </button>

            <!-- 4. Folder: SFTP File Manager SVG -->
            <a
              href="/sftp?host={host.id}"
              class="w-8 h-8 flex items-center justify-center text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white rounded-lg text-xs font-medium border border-neutral-200 dark:border-neutral-700 hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-all shrink-0"
              title="Files (SFTP)"
              aria-label="Files (SFTP)"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"></path></svg>
            </a>

            <!-- 5. Info: Host Details SVG -->
            <button
              onclick={() => (detailHost = host)}
              class="w-8 h-8 flex items-center justify-center text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white rounded-lg text-xs font-medium border border-neutral-200 dark:border-neutral-700 hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-all shrink-0"
              title="Host Details"
              aria-label="Host Details"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path></svg>
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

{#if detailHost}
  <HostDetailPanel host={detailHost} onClose={() => (detailHost = null)} />
{/if}

<!-- Add Host Modal -->
{#if isAddModalOpen}
  <div class="fixed inset-0 bg-black/60 dark:bg-black/80 flex items-center justify-center p-4 z-50">
    <div class="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-xl p-6 max-w-lg w-full space-y-4 shadow-2xl">
      <div class="flex items-center justify-between border-b border-neutral-200 dark:border-neutral-800 pb-3">
        <h3 class="text-lg font-bold text-neutral-900 dark:text-white">{editingId ? 'Edit SSH Host' : 'Add New SSH Host'}</h3>
        <button onclick={() => isAddModalOpen = false} class="text-neutral-400 hover:text-neutral-600 dark:hover:text-neutral-300 text-xl font-bold">×</button>
      </div>

      {#if errorMsg}
        <div class="p-3 text-xs rounded bg-rose-500/10 border border-rose-500/30 text-rose-500 dark:text-rose-400">
          {errorMsg}
        </div>
      {/if}

      <div class="space-y-4 text-sm">
        <div>
          <label class="block text-xs font-medium text-neutral-700 dark:text-neutral-400 mb-1">Host Label / Name *</label>
          <input 
            type="text" 
            bind:value={formLabel} 
            placeholder="Production VPS / YPC Server"
            class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded-lg text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500 shadow-sm dark:shadow-none"
          />
        </div>

        <div class="grid grid-cols-3 gap-3">
          <div class="col-span-2">
            <label class="block text-xs font-medium text-neutral-700 dark:text-neutral-400 mb-1">Host / IP Address *</label>
            <input 
              type="text" 
              bind:value={formAddress} 
              placeholder="100.76.150.46 or vps.domain.com"
              class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded-lg text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500 shadow-sm dark:shadow-none"
            />
          </div>
          <div>
            <label class="block text-xs font-medium text-neutral-700 dark:text-neutral-400 mb-1">Port</label>
            <input 
              type="number" 
              bind:value={formPort} 
              placeholder="22"
              class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded-lg text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500 shadow-sm dark:shadow-none"
            />
          </div>
        </div>

        <div>
          <label class="block text-xs font-medium text-neutral-700 dark:text-neutral-400 mb-1">Username *</label>
          <input 
            type="text" 
            bind:value={formUsername} 
            placeholder="root / cecep"
            class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded-lg text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500 shadow-sm dark:shadow-none"
          />
        </div>

        <div>
          <label class="block text-xs font-medium text-neutral-700 dark:text-neutral-400 mb-1">Authentication Method</label>
          <div class="flex gap-4 mb-2">
            <label class="flex items-center gap-2 text-xs text-neutral-700 dark:text-neutral-300">
              <input type="radio" value="password" bind:group={formAuthType} class="text-sky-600 focus:ring-0" />
              Password
            </label>
            <label class="flex items-center gap-2 text-xs text-neutral-700 dark:text-neutral-300">
              <input type="radio" value="keyId" bind:group={formAuthType} class="text-sky-600 focus:ring-0" />
              Vault Key
            </label>
            <label class="flex items-center gap-2 text-xs text-neutral-700 dark:text-neutral-300">
              <input type="radio" value="key" bind:group={formAuthType} class="text-sky-600 focus:ring-0" />
              Local File
            </label>
          </div>
          {#if formAuthType === 'key'}
            <input
              type="text"
              bind:value={formKeyPath}
              placeholder="/home/user/.ssh/id_ed25519"
              class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded-lg text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500 mb-3 shadow-sm dark:shadow-none"
            />
          {:else if formAuthType === 'keyId'}
            <select
              bind:value={formKeyId}
              class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded-lg text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500 mb-3 shadow-sm dark:shadow-none"
            >
              <option value="" disabled>-- Select a key from vault --</option>
              {#each vaultKeys as k}
                <option value={k.id}>{k.name} ({k.algorithm})</option>
              {/each}
            </select>
          {/if}
          {#if formAuthType !== 'keyId'}
          <label class="block text-xs font-medium text-neutral-700 dark:text-neutral-400 mb-1">
            {formAuthType === 'key' ? 'Key Passphrase (optional)' : 'Password'}
          </label>
          <div class="relative">
            <input
              type={showFormSecret ? 'text' : 'password'}
              bind:value={formSecret}
              placeholder={editingId && editingHadSecret ? 'Leave blank to keep unchanged' : (formAuthType === 'key' ? 'Leave blank if key has no passphrase' : 'Required to connect')}
              class="w-full pl-3 pr-10 py-2 bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded-lg text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500 shadow-sm dark:shadow-none"
            />
            <button
              type="button"
              onclick={() => (showFormSecret = !showFormSecret)}
              class="absolute right-2.5 top-1/2 -translate-y-1/2 text-neutral-400 hover:text-neutral-600 dark:hover:text-neutral-200 p-1"
              aria-label={showFormSecret ? 'Hide secret' : 'Show secret'}
              title={showFormSecret ? 'Hide secret' : 'Show secret'}
            >
              {#if showFormSecret}
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21" />
                </svg>
              {:else}
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                </svg>
              {/if}
            </button>
          </div>
          {/if}
          <p class="text-neutral-500 text-xs mt-1">Stored encrypted (AES-256-GCM) in local database — never returned to the UI.</p>
        </div>

        <div>
          <label class="block text-xs font-medium text-neutral-700 dark:text-neutral-400 mb-1">Operating System / Distro (Optional)</label>
          <div class="flex items-center gap-2">
            <OsIcon os={formOs} name={formLabel} tags={formTags.split(',')} address={formAddress} size={20} />
            <select
              bind:value={formOs}
              class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded-lg text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500 shadow-sm dark:shadow-none text-xs"
            >
              <option value="">Auto Detect (from Name, Tags, or Address)</option>
              <optgroup label="Popular Linux Distros">
                <option value="ubuntu">Ubuntu</option>
                <option value="debian">Debian</option>
                <option value="fedora">Fedora</option>
                <option value="redhat">Red Hat / RHEL</option>
                <option value="centos">CentOS</option>
                <option value="rocky">Rocky Linux</option>
                <option value="almalinux">AlmaLinux</option>
                <option value="arch">Arch Linux</option>
                <option value="manjaro">Manjaro</option>
                <option value="alpine">Alpine Linux</option>
                <option value="opensuse">openSUSE / SUSE</option>
                <option value="mint">Linux Mint</option>
                <option value="kali">Kali Linux</option>
                <option value="popos">Pop!_OS</option>
                <option value="gentoo">Gentoo</option>
                <option value="void">Void Linux</option>
                <option value="nixos">NixOS</option>
                <option value="endeavour">EndeavourOS</option>
                <option value="elementary">Elementary OS</option>
                <option value="zorin">Zorin OS</option>
                <option value="raspberry">Raspberry Pi OS / Raspbian</option>
                <option value="amazon">Amazon Linux</option>
                <option value="oracle">Oracle Linux</option>
                <option value="slackware">Slackware</option>
                <option value="mageia">Mageia</option>
                <option value="solus">Solus</option>
                <option value="tails">Tails</option>
                <option value="deepin">Deepin</option>
                <option value="clear">Clear Linux</option>
                <option value="garuda">Garuda Linux</option>
                <option value="steam">SteamOS</option>
                <option value="coreos">CoreOS / Flatcar</option>
                <option value="devuan">Devuan</option>
                <option value="parrot">Parrot OS</option>
                <option value="mx">MX Linux</option>
                <option value="lubuntu">Lubuntu / Xubuntu / Kubuntu</option>
                <option value="linux">Generic Linux (Tux)</option>
              </optgroup>
              <optgroup label="Other Operating Systems &amp; Devices">
                <option value="windows">Windows</option>
                <option value="macos">macOS / Apple</option>
                <option value="android">Android</option>
                <option value="ios">iOS</option>
                <option value="freebsd">FreeBSD</option>
                <option value="openbsd">OpenBSD</option>
                <option value="netbsd">NetBSD</option>
                <option value="mikrotik">MikroTik / RouterOS</option>
                <option value="cisco">Cisco IOS</option>
                <option value="server">Generic Server</option>
              </optgroup>
            </select>
          </div>
        </div>

        <div>
          <label class="block text-xs font-medium text-neutral-700 dark:text-neutral-400 mb-1">Tags (Comma-separated)</label>
          <input 
            type="text" 
            bind:value={formTags} 
            placeholder="prod, vps, tailscale"
            class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded-lg text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500 shadow-sm dark:shadow-none"
          />
        </div>
      </div>

      <div class="flex justify-end gap-3 pt-4 border-t border-neutral-200 dark:border-neutral-800">
        <button 
          onclick={() => isAddModalOpen = false} 
          class="px-4 py-2 bg-neutral-100 dark:bg-neutral-800 hover:bg-neutral-200 dark:hover:bg-neutral-700 text-neutral-700 dark:text-neutral-300 rounded-lg text-sm font-medium transition-colors">
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
