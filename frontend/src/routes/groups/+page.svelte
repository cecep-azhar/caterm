<script lang="ts">
  import { onMount } from 'svelte';
  import { openSession } from '$lib/nav';
  import { listGroups, saveGroup, deleteGroup, type GroupRecord } from '$lib/api/groups';
  import { listHosts, type HostRecord } from '$lib/api/hosts';
  import { listSnippets, type SnippetRecord } from '$lib/api/snippets';
  import { invoke } from '@tauri-apps/api/core';
  import { t } from '$lib/i18n/index.svelte';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import { showToast, confirmModal } from '$lib/stores/uiNotifications.svelte';

  type ResourceTab = 'all' | 'hosts' | 'snippets' | 'totp' | 'keys' | 'tunnels' | 'investigations';

  let isAddModalOpen = $state(false);
  let backendAvailable = $state(true);
  let groups = $state<GroupRecord[]>([]);
  let availableHosts = $state<HostRecord[]>([]);
  let availableSnippets = $state<SnippetRecord[]>([]);
  let availableTotp = $state<any[]>([]);
  let availableKeys = $state<any[]>([]);
  let availableTunnels = $state<any[]>([]);
  let availableInvestigations = $state<any[]>([]);
  let activeTab = $state<ResourceTab>('all');
  let searchQuery = $state('');

  let editingId = $state<string | null>(null);

  let newGroup = $state({
    name: '',
    color: '#3b82f6',
    selectedHosts: [] as string[]
  });

  const colors = ['#ef4444', '#f97316', '#f59e0b', '#10b981', '#06b6d4', '#3b82f6', '#8b5cf6', '#ec4899'];

  async function loadAllData() {
    try {
      const [g, h, s] = await Promise.all([
        listGroups(),
        listHosts(),
        listSnippets()
      ]);
      groups = g;
      availableHosts = h;
      availableSnippets = s;

      // Optional async fetches
      invoke('list_totp_entries').then((res: any) => { availableTotp = res || []; }).catch(() => {});
      invoke('list_keys').then((res: any) => { availableKeys = res || []; }).catch(() => {});
      invoke('list_tunnels').then((res: any) => { availableTunnels = res || []; }).catch(() => {});
      invoke('list_investigations').then((res: any) => { availableInvestigations = res || []; }).catch(() => {});
      backendAvailable = true;
    } catch {
      backendAvailable = false;
    }
  }

  onMount(async () => {
    await loadAllData();
    const handleUpdate = () => {
      void loadAllData();
    };
    window.addEventListener('caterm:groups-updated', handleUpdate);
    window.addEventListener('caterm:hosts-updated', handleUpdate);
    window.addEventListener('caterm:snippets-updated', handleUpdate);
    return () => {
      window.removeEventListener('caterm:groups-updated', handleUpdate);
      window.removeEventListener('caterm:hosts-updated', handleUpdate);
      window.removeEventListener('caterm:snippets-updated', handleUpdate);
    };
  });

  function hostLabel(id: string) {
    return availableHosts.find((h) => h.id === id)?.label ?? id;
  }

  function getSnippetsForGroup(groupName: string) {
    const q = groupName.toLowerCase();
    return availableSnippets.filter(s => s.tags.some(t => t.toLowerCase() === q) || s.label.toLowerCase().includes(q));
  }

  function getTotpForGroup(groupName: string) {
    const q = groupName.toLowerCase();
    return availableTotp.filter(t => (t.issuer && t.issuer.toLowerCase().includes(q)) || t.label.toLowerCase().includes(q));
  }

  function getTunnelsForGroup(group: GroupRecord) {
    return availableTunnels.filter(t => group.hostIds.includes(t.hostId) || t.label?.toLowerCase().includes(group.name.toLowerCase()));
  }

  function getInvestigationsForGroup(group: GroupRecord) {
    return availableInvestigations.filter(i => group.hostIds.includes(i.hostId) || i.title?.toLowerCase().includes(group.name.toLowerCase()));
  }

  function openAddModal() {
    editingId = null;
    newGroup = { name: '', color: '#3b82f6', selectedHosts: [] };
    isAddModalOpen = true;
  }

  function openEditModal(group: GroupRecord) {
    editingId = group.id;
    newGroup = { name: group.name, color: group.color, selectedHosts: [...group.hostIds] };
    isAddModalOpen = true;
  }

  async function createGroup(e: Event) {
    e.preventDefault();
    if (!newGroup.name.trim()) return;

    try {
      const saved = await saveGroup({
        id: editingId ?? undefined,
        name: newGroup.name.trim(),
        color: newGroup.color,
        hostIds: newGroup.selectedHosts
      });
      groups = editingId
        ? groups.map((g) => (g.id === saved.id ? saved : g))
        : [...groups, saved];
      showToast(editingId ? 'Grup diperbarui!' : 'Grup berhasil dibuat!', 'success');
      window.dispatchEvent(new CustomEvent('caterm:groups-updated'));
    } catch (err) {
      showToast(String(err), 'error');
    }

    editingId = null;
    newGroup = { name: '', color: '#3b82f6', selectedHosts: [] };
    isAddModalOpen = false;
  }

  async function removeGroup(id: string, name: string) {
    const confirmed = await confirmModal(
      `Apakah Anda yakin ingin menghapus grup "${name}"? Host dan resource di dalamnya tidak akan terhapus.`,
      'Hapus Grup',
      true,
      'Hapus',
      'Batal'
    );
    if (confirmed) {
      try {
        await deleteGroup(id);
        groups = groups.filter((g) => g.id !== id);
        showToast(`Grup "${name}" dihapus!`, 'success');
        window.dispatchEvent(new CustomEvent('caterm:groups-updated'));
      } catch (e: any) {
        showToast(String(e), 'error');
      }
    }
  }

  function toggleHost(hostId: string) {
    if (newGroup.selectedHosts.includes(hostId)) {
      newGroup.selectedHosts = newGroup.selectedHosts.filter(id => id !== hostId);
    } else {
      newGroup.selectedHosts.push(hostId);
    }
  }

  const filteredGroups = $derived.by(() => {
    const q = searchQuery.toLowerCase().trim();
    if (!q) return groups;
    return groups.filter(g => g.name.toLowerCase().includes(q));
  });
</script>

<svelte:head>
  <title>Resource & Workspaces Groups · CATerm</title>
</svelte:head>

<div class="max-w-5xl mx-auto space-y-6">
  <PageHeader
    icon={['M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z']}
    accent="indigo"
    title="Workspaces & Resource Groups"
    subtitle="Kelompokkan Host, Snippets, 2FA, SSH Keys, Port Forwarding, dan Investigasi dalam satu kategori terpadu."
  >
    {#snippet actions()}
      <button
        type="button"
        onclick={openAddModal}
        class="px-4 py-2 bg-sky-600 hover:bg-sky-500 text-white font-semibold text-xs rounded-xl transition-all flex items-center gap-2 shadow-md shadow-sky-600/20">
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"></path></svg>
        {t('groups.createGroup')}
      </button>
    {/snippet}
  </PageHeader>

  {#if !backendAvailable}
    <p class="text-amber-500 text-xs -mt-4">{t('common.backendUnavailable')}</p>
  {/if}

  <!-- Filter & Resource Categories Bar -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 border-b border-neutral-200 dark:border-neutral-800 pb-3">
    <!-- Resource Tabs -->
    <div class="flex items-center gap-1 overflow-x-auto pb-1 text-xs">
      <button
        type="button"
        onclick={() => (activeTab = 'all')}
        class="px-3 py-1.5 rounded-lg font-medium transition-colors {activeTab === 'all' ? 'bg-neutral-900 text-white dark:bg-white dark:text-neutral-900' : 'text-neutral-600 dark:text-neutral-400 hover:bg-neutral-100 dark:hover:bg-neutral-800'}"
      >
        Semua Resource
      </button>
      <button
        type="button"
        onclick={() => (activeTab = 'hosts')}
        class="px-3 py-1.5 rounded-lg font-medium transition-colors {activeTab === 'hosts' ? 'bg-sky-600 text-white' : 'text-neutral-600 dark:text-neutral-400 hover:bg-neutral-100 dark:hover:bg-neutral-800'}"
      >
        🖥️ Hosts ({availableHosts.length})
      </button>
      <button
        type="button"
        onclick={() => (activeTab = 'snippets')}
        class="px-3 py-1.5 rounded-lg font-medium transition-colors {activeTab === 'snippets' ? 'bg-emerald-600 text-white' : 'text-neutral-600 dark:text-neutral-400 hover:bg-neutral-100 dark:hover:bg-neutral-800'}"
      >
        📜 Snippets ({availableSnippets.length})
      </button>
      <button
        type="button"
        onclick={() => (activeTab = 'totp')}
        class="px-3 py-1.5 rounded-lg font-medium transition-colors {activeTab === 'totp' ? 'bg-amber-600 text-white' : 'text-neutral-600 dark:text-neutral-400 hover:bg-neutral-100 dark:hover:bg-neutral-800'}"
      >
        🔐 2FA ({availableTotp.length})
      </button>
      <button
        type="button"
        onclick={() => (activeTab = 'tunnels')}
        class="px-3 py-1.5 rounded-lg font-medium transition-colors {activeTab === 'tunnels' ? 'bg-indigo-600 text-white' : 'text-neutral-600 dark:text-neutral-400 hover:bg-neutral-100 dark:hover:bg-neutral-800'}"
      >
        🔀 Tunnels ({availableTunnels.length})
      </button>
      <button
        type="button"
        onclick={() => (activeTab = 'investigations')}
        class="px-3 py-1.5 rounded-lg font-medium transition-colors {activeTab === 'investigations' ? 'bg-purple-600 text-white' : 'text-neutral-600 dark:text-neutral-400 hover:bg-neutral-100 dark:hover:bg-neutral-800'}"
      >
        🔍 Investigations ({availableInvestigations.length})
      </button>
    </div>

    <!-- Search Input -->
    <div class="relative w-full sm:w-64">
      <input
        type="text"
        bind:value={searchQuery}
        placeholder="Cari grup workspace..."
        class="w-full pl-8 pr-3 py-1.5 text-xs rounded-xl bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 text-neutral-900 dark:text-white placeholder-neutral-400 focus:outline-none focus:border-sky-500"
      />
      <svg class="w-3.5 h-3.5 text-neutral-400 absolute left-2.5 top-2.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
      </svg>
    </div>
  </div>

  <!-- Groups Grid -->
  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
    {#each filteredGroups as group (group.id)}
      {@const groupSnippets = getSnippetsForGroup(group.name)}
      {@const groupTotp = getTotpForGroup(group.name)}
      {@const groupTunnels = getTunnelsForGroup(group)}
      {@const groupInvs = getInvestigationsForGroup(group)}

      <div class="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-2xl p-5 flex flex-col justify-between hover:border-neutral-300 dark:hover:border-neutral-700 transition-all shadow-xs group">
        <div class="space-y-3">
          <!-- Card Header -->
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2.5">
              <span class="w-3.5 h-3.5 rounded-full inline-block shrink-0 shadow-xs" style="background-color: {group.color}"></span>
              <h3 class="font-bold text-neutral-900 dark:text-white text-base truncate">{group.name}</h3>
            </div>
            {#if group.hostIds.length > 0}
              <button
                type="button"
                onclick={() => openSession(group.hostIds)}
                class="px-2.5 py-1 bg-sky-500/10 hover:bg-sky-500 text-sky-600 dark:text-sky-400 hover:text-white rounded-lg text-xs font-semibold transition-colors flex items-center gap-1"
                title="Buka semua sesi terminal host di grup ini"
              >
                <span>⚡ {t('groups.launchAll')}</span>
              </button>
            {/if}
          </div>

          <!-- Multi-Resource Summary Counters -->
          <div class="grid grid-cols-3 gap-1.5 text-center text-[11px] py-1 bg-neutral-50 dark:bg-neutral-950/60 rounded-xl border border-neutral-100 dark:border-neutral-800/60">
            <div>
              <span class="block font-bold text-neutral-900 dark:text-white">{group.hostIds.length}</span>
              <span class="text-[10px] text-neutral-500">Hosts</span>
            </div>
            <div>
              <span class="block font-bold text-neutral-900 dark:text-white">{groupSnippets.length}</span>
              <span class="text-[10px] text-neutral-500">Snippets</span>
            </div>
            <div>
              <span class="block font-bold text-neutral-900 dark:text-white">{groupTotp.length}</span>
              <span class="text-[10px] text-neutral-500">2FA Tokens</span>
            </div>
          </div>

          <!-- Assigned Hosts List -->
          {#if (activeTab === 'all' || activeTab === 'hosts') && group.hostIds.length > 0}
            <div>
              <span class="text-[10px] font-semibold uppercase tracking-wider text-neutral-400 block mb-1">Assigned Hosts</span>
              <div class="flex gap-1 flex-wrap">
                {#each group.hostIds as hostId}
                  <span class="px-2 py-0.5 bg-neutral-100 dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700/80 text-neutral-700 dark:text-neutral-300 rounded-md text-xs font-mono">
                    {hostLabel(hostId)}
                  </span>
                {/each}
              </div>
            </div>
          {/if}

          <!-- Assigned Snippets -->
          {#if (activeTab === 'all' || activeTab === 'snippets') && groupSnippets.length > 0}
            <div>
              <span class="text-[10px] font-semibold uppercase tracking-wider text-emerald-500 dark:text-emerald-400 block mb-1">Snippets & Commands</span>
              <div class="space-y-1 max-h-24 overflow-y-auto">
                {#each groupSnippets as snip}
                  <div class="px-2 py-1 bg-emerald-500/5 border border-emerald-500/20 rounded-md text-[11px] flex items-center justify-between">
                    <span class="font-medium truncate text-neutral-900 dark:text-white">{snip.label}</span>
                    <code class="text-[10px] text-neutral-500 truncate max-w-[100px]">{snip.command}</code>
                  </div>
                {/each}
              </div>
            </div>
          {/if}

          <!-- Assigned 2FA Tokens -->
          {#if (activeTab === 'all' || activeTab === 'totp') && groupTotp.length > 0}
            <div>
              <span class="text-[10px] font-semibold uppercase tracking-wider text-amber-500 dark:text-amber-400 block mb-1">2FA Authenticator Tokens</span>
              <div class="flex gap-1 flex-wrap">
                {#each groupTotp as totp}
                  <span class="px-2 py-0.5 bg-amber-500/10 border border-amber-500/20 text-amber-700 dark:text-amber-300 rounded-md text-[11px]">
                    🔑 {totp.label}
                  </span>
                {/each}
              </div>
            </div>
          {/if}
        </div>

        <!-- Action Footer -->
        <div class="mt-4 pt-3 border-t border-neutral-100 dark:border-neutral-800/80 flex items-center justify-between text-xs">
          <span class="text-[11px] text-neutral-400">
            {group.hostIds.length + groupSnippets.length + groupTotp.length} Item Total
          </span>
          <div class="flex gap-1.5">
            <button
              type="button"
              onclick={() => openEditModal(group)}
              class="px-2.5 py-1 bg-neutral-100 dark:bg-neutral-800 hover:bg-sky-600 hover:text-white rounded-lg text-neutral-700 dark:text-neutral-300 transition-colors"
            >
              {t('common.edit')}
            </button>
            <button
              type="button"
              onclick={() => removeGroup(group.id, group.name)}
              class="px-2.5 py-1 bg-neutral-100 dark:bg-neutral-800 hover:bg-rose-600 hover:text-white rounded-lg text-neutral-700 dark:text-neutral-300 transition-colors"
            >
              {t('common.delete')}
            </button>
          </div>
        </div>
      </div>
    {/each}
  </div>
</div>

<!-- Create / Edit Group Modal -->
{#if isAddModalOpen}
  <div class="fixed inset-0 bg-black/60 dark:bg-black/80 flex items-center justify-center p-4 z-50 backdrop-blur-xs">
    <div class="bg-white dark:bg-[#141414] border border-neutral-200 dark:border-neutral-800 rounded-2xl p-6 max-w-lg w-full space-y-4 shadow-2xl animate-in fade-in zoom-in-95 duration-150">
      <h3 class="text-lg font-bold text-neutral-900 dark:text-white">{editingId ? t('groups.editGroup') : t('groups.createGroup')}</h3>
      
      <form onsubmit={createGroup} class="space-y-4 text-xs">
        <div>
          <label for="group-name" class="block font-medium text-neutral-700 dark:text-neutral-300 mb-1">Nama Grup Workspace *</label>
          <input id="group-name" bind:value={newGroup.name} required placeholder="misal: Production VPS / Client Riset / Gerlink Store" class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 rounded-lg text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500" />
        </div>

        <div>
          <span class="block font-medium text-neutral-700 dark:text-neutral-300 mb-2">{t('groups.color')}</span>
          <div class="flex gap-2">
            {#each colors as c}
              <button 
                type="button"
                onclick={() => (newGroup.color = c)}
                class="w-7 h-7 rounded-full transition-transform border-2 {newGroup.color === c ? 'border-white scale-110 shadow-md' : 'border-transparent'}" 
                style="background-color: {c}"
                aria-label={t('common.selectColor', { color: c })}>
              </button>
            {/each}
          </div>
        </div>

        <div>
          <span class="block font-medium text-neutral-700 dark:text-neutral-300 mb-1">{t('groups.hostsInGroup')}</span>
          <div class="space-y-1 max-h-48 overflow-y-auto p-2 bg-neutral-50 dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-lg">
            {#each availableHosts as h}
              <label class="flex items-center gap-2 p-1.5 hover:bg-neutral-100 dark:hover:bg-neutral-800 rounded-md cursor-pointer text-neutral-700 dark:text-neutral-300">
                <input 
                  type="checkbox" 
                  checked={newGroup.selectedHosts.includes(h.id)} 
                  onchange={() => toggleHost(h.id)}
                  class="rounded bg-white dark:bg-neutral-900 border-neutral-300 dark:border-neutral-700 text-sky-600 focus:ring-sky-500"
                />
                <span class="font-medium">{h.label}</span>
                <span class="text-[10px] text-neutral-400 font-mono ml-auto">{h.address}</span>
              </label>
            {/each}
          </div>
          <p class="text-[11px] text-neutral-500 mt-1">
            Snippet dan 2FA token dengan tag/nama yang cocok akan otomatis terkategori ke dalam grup ini.
          </p>
        </div>

        <div class="flex justify-end gap-2 pt-3 border-t border-neutral-200 dark:border-neutral-800">
          <button 
            type="button"
            onclick={() => (isAddModalOpen = false)} 
            class="px-4 py-2 bg-neutral-100 dark:bg-neutral-800 hover:bg-neutral-200 dark:hover:bg-neutral-700 text-neutral-700 dark:text-neutral-300 rounded-lg font-semibold transition-colors">
            {t('common.cancel')}
          </button>
          <button 
            type="submit"
            class="px-4 py-2 bg-sky-600 hover:bg-sky-500 text-white rounded-lg font-semibold shadow-md shadow-sky-600/20 transition-colors">
            {t('groups.saveGroup')}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}
