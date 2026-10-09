<script lang="ts">
  import { onMount } from 'svelte';
  import { openSession } from '$lib/nav';
  import { listGroups, saveGroup, deleteGroup, type GroupRecord } from '$lib/api/groups';
  import { listHosts, type HostRecord } from '$lib/api/hosts';
  import { listSnippets, type SnippetRecord } from '$lib/api/snippets';
  import { listScheduledTasks, type ScheduledTaskRecord } from '$lib/api/tasks';
  import { invoke } from '@tauri-apps/api/core';
  import { t } from '$lib/i18n/index.svelte';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import Pagination from '$lib/components/Pagination.svelte';
  import { showToast, confirmModal } from '$lib/stores/uiNotifications.svelte';
  import { copyText } from '$lib/utils/clipboard';

  const ALL_CATEGORIES = [
    { id: 'hosts', label: 'Hosts & Servers', icon: '🖥️' },
    { id: 'snippets', label: 'Command Snippets', icon: '📜' },
    { id: 'tasks', label: 'Scheduled Tasks', icon: '⏰' },
    { id: 'tunnels', label: 'Port Forwarding', icon: '🔀' },
    { id: 'keys', label: 'SSH Keys', icon: '🔑' },
    { id: 'totp', label: '2FA Authenticator', icon: '🔐' },
    { id: 'investigations', label: 'Investigations', icon: '🔍' },
    { id: 'teams', label: 'Teams', icon: '👥' }
  ];

  let isAddModalOpen = $state(false);
  const isTauri = typeof window !== 'undefined' && ('__TAURI_INTERNALS__' in window || '__TAURI__' in window);
  let backendAvailable = $state(isTauri);
  let groups = $state<GroupRecord[]>([]);
  let availableHosts = $state<HostRecord[]>([]);
  let availableSnippets = $state<SnippetRecord[]>([]);
  let availableTasks = $state<ScheduledTaskRecord[]>([]);
  let availableTotp = $state<any[]>([]);
  let availableKeys = $state<any[]>([]);
  let availableTunnels = $state<any[]>([]);
  let availableInvestigations = $state<any[]>([]);

  let activeCategoryFilter = $state('all');
  let searchQuery = $state('');
  let currentPage = $state(1);
  let pageSize = $state(10);

  let editingId = $state<string | null>(null);
  let formName = $state('');
  let formColor = $state('#3b82f6');
  let formSelectedHosts = $state<string[]>([]);
  let formSelectedCategories = $state<string[]>(['hosts', 'snippets']);

  const colors = ['#ef4444', '#f97316', '#f59e0b', '#10b981', '#06b6d4', '#3b82f6', '#8b5cf6', '#ec4899'];

  async function loadAllData() {
    backendAvailable = typeof window !== 'undefined' && ('__TAURI_INTERNALS__' in window || '__TAURI__' in window);
    const results = await Promise.allSettled([
      listGroups(),
      listHosts(),
      listSnippets(),
      listScheduledTasks().catch(() => [])
    ]);

    if (results[0].status === 'fulfilled') {
      groups = results[0].value || [];
    } else {
      console.error('Failed to load groups:', results[0].reason);
    }

    if (results[1].status === 'fulfilled') {
      availableHosts = results[1].value || [];
    } else {
      console.error('Failed to load hosts:', results[1].reason);
    }

    if (results[2].status === 'fulfilled') {
      availableSnippets = results[2].value || [];
    } else {
      console.error('Failed to load snippets:', results[2].reason);
    }

    if (results[3].status === 'fulfilled') {
      availableTasks = results[3].value || [];
    } else {
      console.error('Failed to load scheduled tasks:', results[3].reason);
    }

    invoke('list_totp_entries').then((res: any) => { availableTotp = res || []; }).catch(() => {});
    invoke('list_keys').then((res: any) => { availableKeys = res || []; }).catch(() => {});
    invoke('list_tunnels').then((res: any) => { availableTunnels = res || []; }).catch(() => {});
    invoke('list_investigations').then((res: any) => { availableInvestigations = res || []; }).catch(() => {});
  }

  onMount(async () => {
    await loadAllData();
    const handleUpdate = () => { void loadAllData(); };
    window.addEventListener('caterm:groups-updated', handleUpdate);
    window.addEventListener('caterm:hosts-updated', handleUpdate);
    window.addEventListener('caterm:snippets-updated', handleUpdate);
    window.addEventListener('caterm:tasks-updated', handleUpdate);
    return () => {
      window.removeEventListener('caterm:groups-updated', handleUpdate);
      window.removeEventListener('caterm:hosts-updated', handleUpdate);
      window.removeEventListener('caterm:snippets-updated', handleUpdate);
      window.removeEventListener('caterm:tasks-updated', handleUpdate);
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

  function openAddModal() {
    editingId = null;
    formName = '';
    formColor = '#3b82f6';
    formSelectedHosts = [];
    formSelectedCategories = ['hosts', 'snippets', 'tasks'];
    isAddModalOpen = true;
  }

  function openEditModal(group: GroupRecord) {
    editingId = group.id;
    formName = group.name;
    formColor = group.color;
    formSelectedHosts = [...group.hostIds];
    formSelectedCategories = group.categories && group.categories.length > 0 ? [...group.categories] : ['hosts', 'snippets'];
    isAddModalOpen = true;
  }

  function toggleCategory(catId: string) {
    if (formSelectedCategories.includes(catId)) {
      formSelectedCategories = formSelectedCategories.filter(c => c !== catId);
    } else {
      formSelectedCategories = [...formSelectedCategories, catId];
    }
  }

  function toggleHost(hostId: string) {
    if (formSelectedHosts.includes(hostId)) {
      formSelectedHosts = formSelectedHosts.filter(id => id !== hostId);
    } else {
      formSelectedHosts = [...formSelectedHosts, hostId];
    }
  }

  async function handleSaveGroup(e: Event) {
    e.preventDefault();
    if (!formName.trim()) return;

    try {
      const saved = await saveGroup({
        id: editingId ?? undefined,
        name: formName.trim(),
        color: formColor,
        hostIds: formSelectedHosts,
        categories: formSelectedCategories.length > 0 ? formSelectedCategories : ['hosts']
      });

      groups = editingId
        ? groups.map((g) => (g.id === saved.id ? saved : g))
        : [...groups, saved];

      showToast(editingId ? 'Grup kerja diperbarui!' : 'Grup kerja berhasil dibuat!', 'success');
      window.dispatchEvent(new CustomEvent('caterm:groups-updated'));
    } catch (err) {
      showToast(String(err), 'error');
    }

    editingId = null;
    isAddModalOpen = false;
  }

  async function removeGroup(id: string, name: string) {
    const confirmed = await confirmModal(
      `Hapus grup kerja "${name}"? Seluruh resource di dalamnya akan tetap aman.`,
      'Hapus Grup Kerja',
      true,
      'Hapus',
      'Batal'
    );
    if (confirmed) {
      try {
        await deleteGroup(id);
        groups = groups.filter((g) => g.id !== id);
        showToast(`Grup kerja "${name}" dihapus!`, 'success');
        window.dispatchEvent(new CustomEvent('caterm:groups-updated'));
      } catch (e: any) {
        showToast(String(e), 'error');
      }
    }
  }

  function exportGroupsJson() {
    const jsonStr = JSON.stringify(groups, null, 2);
    const blob = new Blob([jsonStr], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `caterm-work-groups-${new Date().toISOString().slice(0, 10)}.json`;
    a.click();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
    showToast('Data Work Groups diekspor!', 'success');
  }

  function importGroupsJson() {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.json';
    input.onchange = async (e: any) => {
      const file = e.target?.files?.[0];
      if (!file) return;
      try {
        const text = await file.text();
        const imported = JSON.parse(text);
        if (Array.isArray(imported)) {
          for (const item of imported) {
            if (item.name) {
              await saveGroup({
                name: item.name,
                color: item.color || '#3b82f6',
                hostIds: item.hostIds || [],
                categories: item.categories || ['hosts']
              });
            }
          }
          showToast(`${imported.length} Grup Kerja berhasil diimpor!`, 'success');
          await loadAllData();
        }
      } catch (err) {
        showToast('Format file JSON tidak valid', 'error');
      }
    };
    input.click();
  }

  const filteredGroups = $derived.by(() => {
    let list = groups;
    if (activeCategoryFilter !== 'all') {
      list = list.filter(g => (g.categories || ['hosts']).includes(activeCategoryFilter));
    }
    const q = searchQuery.toLowerCase().trim();
    if (!q) return list;
    return list.filter(g => g.name.toLowerCase().includes(q));
  });

  const paginatedGroups = $derived.by(() => {
    const start = (currentPage - 1) * pageSize;
    return filteredGroups.slice(start, start + pageSize);
  });
</script>

<svelte:head>
  <title>Work Groups · CATerm</title>
</svelte:head>

<div class="max-w-5xl mx-auto space-y-6">
  <PageHeader
    icon={['M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10']}
    accent="indigo"
    title="Work Groups"
    subtitle="Pengelompokan klaster kerja multi-kategori untuk Hosts, Snippets, Tasks, Port Forwarding, Keys, 2FA, dan Tim."
  >
    {#snippet actions()}
      <div class="flex items-center gap-2">
        <button
          type="button"
          onclick={importGroupsJson}
          class="px-3 py-1.5 rounded-xl border border-neutral-200 dark:border-neutral-800 text-xs font-semibold text-neutral-700 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors"
          title="Impor Grup Kerja dari JSON"
        >
          📥 Impor
        </button>
        <button
          type="button"
          onclick={exportGroupsJson}
          class="px-3 py-1.5 rounded-xl border border-neutral-200 dark:border-neutral-800 text-xs font-semibold text-neutral-700 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors"
          title="Ekspor Grup Kerja ke JSON"
        >
          📤 Ekspor
        </button>
        <button
          type="button"
          onclick={openAddModal}
          class="px-4 py-2 bg-sky-600 hover:bg-sky-500 text-white font-semibold text-xs rounded-xl transition-all flex items-center gap-1.5 shadow-md shadow-sky-600/20"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
          </svg>
          + Buat Work Group
        </button>
      </div>
    {/snippet}
  </PageHeader>

  {#if !backendAvailable}
    <p class="text-amber-500 text-xs -mt-4">{t('common.backendUnavailable')}</p>
  {/if}

  <!-- Category Filter & Search Bar -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 border-b border-neutral-200 dark:border-neutral-800 pb-3">
    <!-- Category Tabs -->
    <div class="flex items-center gap-1.5 overflow-x-auto pb-1 text-xs">
      <button
        type="button"
        onclick={() => { activeCategoryFilter = 'all'; currentPage = 1; }}
        class="px-3 py-1.5 rounded-lg font-medium transition-colors {activeCategoryFilter === 'all' ? 'bg-neutral-900 text-white dark:bg-white dark:text-neutral-900' : 'text-neutral-600 dark:text-neutral-400 hover:bg-neutral-100 dark:hover:bg-neutral-800'}"
      >
        Semua Kategori
      </button>
      {#each ALL_CATEGORIES as cat}
        <button
          type="button"
          onclick={() => { activeCategoryFilter = cat.id; currentPage = 1; }}
          class="px-2.5 py-1.5 rounded-lg font-medium transition-colors flex items-center gap-1.5 {activeCategoryFilter === cat.id ? 'bg-sky-600 text-white' : 'text-neutral-600 dark:text-neutral-400 hover:bg-neutral-100 dark:hover:bg-neutral-800'}"
        >
          <span>{cat.icon}</span>
          <span>{cat.label}</span>
        </button>
      {/each}
    </div>

    <!-- Search Box -->
    <div class="relative w-full sm:w-60 shrink-0">
      <input
        type="text"
        bind:value={searchQuery}
        oninput={() => (currentPage = 1)}
        placeholder="Cari Work Group..."
        class="w-full pl-8 pr-3 py-1.5 text-xs rounded-xl bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 text-neutral-900 dark:text-white placeholder-neutral-400 focus:outline-none focus:border-sky-500"
      />
      <svg class="w-3.5 h-3.5 text-neutral-400 absolute left-2.5 top-2.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
      </svg>
    </div>
  </div>

  <!-- Groups Grid -->
  {#if filteredGroups.length === 0}
    <div class="text-center py-16 px-4 bg-white dark:bg-neutral-900/40 rounded-2xl border border-neutral-200 dark:border-neutral-800 space-y-3">
      <div class="w-12 h-12 rounded-full bg-indigo-500/10 text-indigo-500 flex items-center justify-center mx-auto border border-indigo-500/20">
        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
        </svg>
      </div>
      <h3 class="text-sm font-semibold text-neutral-900 dark:text-white">Belum Ada Work Group</h3>
      <p class="text-xs text-neutral-500 max-w-sm mx-auto">
        Buat grup kerja untuk mengkategorikan host, snippet, tugas cron, kunci SSH, 2FA, dan port forwarding dalam satu kesatuan.
      </p>
      <button
        type="button"
        onclick={openAddModal}
        class="mt-2 inline-flex items-center gap-1.5 px-3.5 py-1.5 rounded-lg text-xs font-semibold bg-sky-600 hover:bg-sky-500 text-white transition-colors"
      >
        + Buat Work Group Pertama
      </button>
    </div>
  {:else}
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      {#each paginatedGroups as group (group.id)}
        {@const groupSnippets = getSnippetsForGroup(group.name)}
        {@const groupTotp = getTotpForGroup(group.name)}
        {@const cats = group.categories || ['hosts']}

        <div class="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-2xl p-5 flex flex-col justify-between hover:border-neutral-300 dark:hover:border-neutral-700 transition-all shadow-xs group">
          <div class="space-y-3">
            <!-- Header -->
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
                  title="Buka semua host di grup ini"
                >
                  <span>⚡ Launch</span>
                </button>
              {/if}
            </div>

            <!-- Active Category Badges -->
            <div class="flex gap-1 flex-wrap">
              {#each cats as cId}
                {@const catMeta = ALL_CATEGORIES.find(c => c.id === cId)}
                {#if catMeta}
                  <span class="text-[10px] font-medium px-2 py-0.5 rounded-md bg-neutral-100 dark:bg-neutral-800 text-neutral-600 dark:text-neutral-400 border border-neutral-200 dark:border-neutral-700/80">
                    {catMeta.icon} {catMeta.label}
                  </span>
                {/if}
              {/each}
            </div>

            <!-- Resource Counts Grid -->
            <div class="grid grid-cols-3 gap-1.5 text-center text-[11px] py-1.5 bg-neutral-50 dark:bg-neutral-950/60 rounded-xl border border-neutral-100 dark:border-neutral-800/60">
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

            <!-- Assigned Hosts -->
            {#if group.hostIds.length > 0}
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
          </div>

          <!-- Footer Actions -->
          <div class="mt-4 pt-3 border-t border-neutral-100 dark:border-neutral-800/80 flex items-center justify-between text-xs">
            <span class="text-[11px] text-neutral-400">
              {cats.length} Kategori Aktif
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
                class="px-2.5 py-1 bg-neutral-100 dark:bg-neutral-800 hover:bg-rose-600 hover:text-white text-neutral-700 dark:text-neutral-300 rounded-lg transition-colors"
              >
                {t('common.delete')}
              </button>
            </div>
          </div>
        </div>
      {/each}
    </div>

    <!-- Reusable Pagination Component -->
    <Pagination
      totalItems={filteredGroups.length}
      bind:page={currentPage}
      bind:pageSize
      pageSizeOptions={[9, 18, 36]}
    />
  {/if}
</div>

<!-- Create / Edit Work Group Modal -->
{#if isAddModalOpen}
  <div class="fixed inset-0 bg-black/60 dark:bg-black/80 flex items-center justify-center p-4 z-50 backdrop-blur-xs">
    <div
      role="dialog"
      aria-modal="true"
      class="bg-white dark:bg-[#141414] border border-neutral-200 dark:border-neutral-800 rounded-2xl p-6 max-w-lg w-full max-h-[90vh] overflow-y-auto space-y-4 shadow-2xl animate-in fade-in zoom-in-95 duration-150 text-xs"
    >
      <div class="flex items-center justify-between border-b border-neutral-200 dark:border-neutral-800 pb-3">
        <h3 class="text-base font-bold text-neutral-900 dark:text-white">
          {editingId ? 'Edit Work Group' : 'Buat Work Group Baru'}
        </h3>
        <button
          type="button"
          onclick={() => (isAddModalOpen = false)}
          class="text-neutral-400 hover:text-neutral-600 dark:hover:text-neutral-200 text-lg font-bold"
        >
          ×
        </button>
      </div>
      
      <form onsubmit={handleSaveGroup} class="space-y-4">
        <!-- Name -->
        <div>
          <label class="block font-medium text-neutral-700 dark:text-neutral-300 mb-1">Nama Work Group *</label>
          <input
            bind:value={formName}
            required
            placeholder="misal: Production VPS / Client Riset / Gerlink Store"
            class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 rounded-lg text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500"
          />
        </div>

        <!-- Color -->
        <div>
          <span class="block font-medium text-neutral-700 dark:text-neutral-300 mb-2">{t('groups.color')}</span>
          <div class="flex gap-2">
            {#each colors as c}
              <button 
                type="button"
                onclick={() => (formColor = c)}
                class="w-7 h-7 rounded-full transition-transform border-2 {formColor === c ? 'border-white scale-110 shadow-md' : 'border-transparent'}" 
                style="background-color: {c}"
                aria-label={`Warna ${c}`}>
              </button>
            {/each}
          </div>
        </div>

        <!-- Category Checklist (REQ: Multi-Category Scope) -->
        <div>
          <span class="block font-medium text-neutral-700 dark:text-neutral-300 mb-1">
            Kategori yang Didukung Work Group Ini:
          </span>
          <div class="grid grid-cols-2 gap-1.5 p-2.5 bg-neutral-50 dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-xl">
            {#each ALL_CATEGORIES as cat}
              <label class="flex items-center gap-2 p-1.5 rounded-lg hover:bg-neutral-100 dark:hover:bg-neutral-800 cursor-pointer text-neutral-800 dark:text-neutral-200 select-none">
                <input
                  type="checkbox"
                  checked={formSelectedCategories.includes(cat.id)}
                  onchange={() => toggleCategory(cat.id)}
                  class="rounded bg-white dark:bg-neutral-900 border-neutral-300 dark:border-neutral-700 text-sky-600 focus:ring-sky-500"
                />
                <span class="text-xs">{cat.icon} {cat.label}</span>
              </label>
            {/each}
          </div>
          <p class="text-[11px] text-neutral-500 mt-1">
            Resource pada menu yang dicentang dapat dimasukkan ke dalam grup kerja ini.
          </p>
        </div>

        <!-- Linked Hosts (if hosts category enabled) -->
        {#if formSelectedCategories.includes('hosts')}
          <div>
            <span class="block font-medium text-neutral-700 dark:text-neutral-300 mb-1">Pilih Server Host untuk Grup Ini:</span>
            <div class="space-y-1 max-h-36 overflow-y-auto p-2 bg-neutral-50 dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-lg">
              {#each availableHosts as h}
                <label class="flex items-center gap-2 p-1.5 hover:bg-neutral-100 dark:hover:bg-neutral-800 rounded-md cursor-pointer text-neutral-700 dark:text-neutral-300 select-none">
                  <input 
                    type="checkbox" 
                    checked={formSelectedHosts.includes(h.id)} 
                    onchange={() => toggleHost(h.id)}
                    class="rounded bg-white dark:bg-neutral-900 border-neutral-300 dark:border-neutral-700 text-sky-600 focus:ring-sky-500"
                  />
                  <span class="font-medium">{h.label}</span>
                  <span class="text-[10px] text-neutral-400 font-mono ml-auto">{h.address}</span>
                </label>
              {/each}
            </div>
          </div>
        {/if}

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
            {editingId ? 'Simpan Perubahan' : 'Buat Work Group'}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}
