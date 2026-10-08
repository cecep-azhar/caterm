<script lang="ts">
  import { onMount } from 'svelte';
  import { injectIntoActiveSession, hasActiveSession } from '$lib/stores/activeSession.svelte';
  import { listSnippets, saveSnippet as saveSnippetApi, deleteSnippet as deleteSnippetApi, type SnippetRecord } from '$lib/api/snippets';
  import { listGroups, type GroupRecord } from '$lib/api/groups';
  import { t } from '$lib/i18n/index.svelte';
  import { showToast, confirmModal } from '$lib/stores/uiNotifications.svelte';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import Pagination from '$lib/components/Pagination.svelte';
  import { save, open } from '@tauri-apps/plugin-dialog';
  import { writeTextFile, readTextFile } from '@tauri-apps/plugin-fs';

  let isAddModalOpen = $state(false);
  let creationStep = $state(1); // 1: command, 2: details
  let injectNotice = $state('');
  let backendAvailable = $state(true);
  let editingId = $state<string | null>(null);

  let snippets = $state<SnippetRecord[]>([]);
  let groups = $state<GroupRecord[]>([]);
  let filterGroup = $state('all');
  let searchQuery = $state('');
  let currentPage = $state(1);
  let pageSize = $state(10);

  let newSnippet = $state({
    cmd: '',
    label: '',
    desc: '',
    tags: '',
    selectedGroup: ''
  });

  async function loadData() {
    try {
      const [sList, gList] = await Promise.all([
        listSnippets(),
        listGroups().catch(() => [])
      ]);
      snippets = sList || [];
      groups = gList || [];
      backendAvailable = true;
    } catch {
      backendAvailable = false;
    }
  }

  onMount(() => {
    void loadData();
    const handleHostsChange = () => {
      loadData();
    };
    window.addEventListener('caterm:hosts-updated', handleHostsChange);
    return () => {
      window.removeEventListener('caterm:hosts-updated', handleHostsChange);
    };
  });

  async function exportSnippets() {
    try {
      const allSnippets = await listSnippets();
      const jsonContent = JSON.stringify(allSnippets, null, 2);
      let saved = false;

      try {
        const filePath = await save({
          defaultPath: 'caterm-snippets-export.json',
          filters: [{ name: 'JSON', extensions: ['json'] }]
        });

        if (filePath) {
          await writeTextFile(filePath, jsonContent);
          saved = true;
          showToast(t('snippets.exportSuccess'), 'success');
        } else {
          return;
        }
      } catch {}

      if (!saved) {
        const blob = new Blob([jsonContent], { type: 'application/json;charset=utf-8;' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'caterm-snippets-export.json';
        a.click();
        setTimeout(() => URL.revokeObjectURL(url), 1000);
        showToast(t('snippets.exportSuccess'), 'success');
      }
    } catch (err) {
      showToast(t('snippets.exportFailed'), 'error');
    }
  }

  async function processSnippetsJson(jsonText: string) {
    try {
      const parsed = JSON.parse(jsonText);
      if (!Array.isArray(parsed)) {
        showToast(t('snippets.importInvalidFormat'), 'error');
        return;
      }
      for (const item of parsed) {
        if (item.label && item.command) {
          await saveSnippetApi({
            label: item.label,
            command: item.command,
            description: item.description || '',
            tags: item.tags || []
          });
        }
      }
      showToast(t('snippets.importSuccess', { count: parsed.length }), 'success');
      await loadData();
    } catch {
      showToast(t('snippets.importFailed'), 'error');
    }
  }

  async function importSnippets() {
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: 'JSON', extensions: ['json'] }]
      });
      if (selected && typeof selected === 'string') {
        const content = await readTextFile(selected);
        await processSnippetsJson(content);
        return;
      }
    } catch {}

    const fileInput = document.createElement('input');
    fileInput.type = 'file';
    fileInput.accept = '.json,application/json';
    fileInput.onchange = async (e: Event) => {
      const target = e.target as HTMLInputElement;
      const file = target?.files?.[0];
      if (file) {
        const text = await file.text();
        await processSnippetsJson(text);
      }
    };
    fileInput.click();
  }

  function resetModal() {
    editingId = null;
    creationStep = 1;
    newSnippet = { cmd: '', label: '', desc: '', tags: '', selectedGroup: '' };
    isAddModalOpen = false;
  }

  function openEditModal(snip: SnippetRecord) {
    editingId = snip.id;
    creationStep = 2;
    newSnippet = {
      cmd: snip.command,
      label: snip.label,
      desc: snip.description,
      tags: snip.tags.join(', '),
      selectedGroup: groups.find(g => snip.tags.some(t => t.toLowerCase() === g.name.toLowerCase()))?.name || ''
    };
    isAddModalOpen = true;
  }

  async function handleSaveSnippet(e: Event) {
    e.preventDefault();
    if (!newSnippet.label.trim() || !newSnippet.cmd.trim()) return;

    let parsedTags = newSnippet.tags
      .split(',')
      .map(t => t.trim())
      .filter(t => t.length > 0);

    if (newSnippet.selectedGroup && !parsedTags.includes(newSnippet.selectedGroup)) {
      parsedTags.push(newSnippet.selectedGroup);
    }

    try {
      await saveSnippetApi({
        id: editingId ?? undefined,
        label: newSnippet.label.trim(),
        description: newSnippet.desc.trim(),
        command: newSnippet.cmd.trim(),
        tags: parsedTags
      });
      showToast(editingId ? 'Snippet diperbarui!' : 'Snippet berhasil disimpan!', 'success');
      window.dispatchEvent(new CustomEvent('caterm:snippets-updated'));
      await loadData();
    } catch {
      backendAvailable = false;
    }

    resetModal();
  }

  async function removeSnippet(id: string, label: string) {
    const confirmed = await confirmModal(
      `Hapus snippet "${label}"?`,
      'Hapus Snippet',
      true,
      'Hapus',
      'Batal'
    );
    if (confirmed) {
      try {
        await deleteSnippetApi(id);
        snippets = snippets.filter(s => s.id !== id);
        showToast(`Snippet "${label}" dihapus!`, 'success');
        window.dispatchEvent(new CustomEvent('caterm:snippets-updated'));
      } catch {
        backendAvailable = false;
      }
    }
  }

  function injectSnippet(cmd: string) {
    const injected = injectIntoActiveSession(cmd);
    injectNotice = injected ? t('snippets.injectedNotice') : t('snippets.noActiveSessionNotice');
    setTimeout(() => (injectNotice = ''), 3000);
  }

  const snippetGroups = $derived(
    groups.filter(g => (g.categories || ['hosts']).includes('snippets'))
  );

  const filteredSnippets = $derived.by(() => {
    let list = snippets;
    if (filterGroup !== 'all') {
      const q = filterGroup.toLowerCase();
      list = list.filter(s => s.tags.some(t => t.toLowerCase() === q) || s.label.toLowerCase().includes(q));
    }
    const q = searchQuery.toLowerCase().trim();
    if (!q) return list;
    return list.filter(s => s.label.toLowerCase().includes(q) || s.command.toLowerCase().includes(q) || s.description.toLowerCase().includes(q));
  });

  const paginatedSnippets = $derived.by(() => {
    const start = (currentPage - 1) * pageSize;
    return filteredSnippets.slice(start, start + pageSize);
  });
</script>

<svelte:head>
  <title>{t('snippets.title')} · CATerm</title>
</svelte:head>

<div class="max-w-5xl mx-auto space-y-6">
  <PageHeader
    icon={['M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4']}
    accent="emerald"
    title={t('snippets.title')}
    subtitle={t('snippets.subtitle')}
  >
    {#snippet actions()}
      <div class="flex items-center gap-2">
        <button
          type="button"
          onclick={importSnippets}
          class="px-3 py-1.5 rounded-xl border border-neutral-200 dark:border-neutral-800 text-xs font-semibold text-neutral-700 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors flex items-center gap-1.5 shadow-2xs"
          title={t('snippets.import')}
        >
          📥 {t('snippets.import')}
        </button>
        <button
          type="button"
          onclick={exportSnippets}
          disabled={snippets.length === 0}
          class="px-3 py-1.5 rounded-xl border border-neutral-200 dark:border-neutral-800 text-xs font-semibold text-neutral-700 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800 disabled:opacity-40 transition-colors flex items-center gap-1.5 shadow-2xs"
          title={t('snippets.export')}
        >
          📤 {t('snippets.export')}
        </button>
        <button
          type="button"
          onclick={() => { resetModal(); isAddModalOpen = true; }}
          class="px-4 py-2 bg-sky-600 hover:bg-sky-500 text-white font-semibold text-xs rounded-xl transition-all flex items-center gap-1.5 shadow-md shadow-sky-600/20"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"></path></svg>
          {t('snippets.addSnippet')}
        </button>
      </div>
    {/snippet}
  </PageHeader>

  {#if !backendAvailable}
    <p class="text-amber-500 text-xs -mt-4">{t('common.backendUnavailable')}</p>
  {/if}

  {#if injectNotice}
    <div class="px-4 py-2 bg-emerald-500/10 border border-emerald-500/30 rounded-xl text-xs text-emerald-600 dark:text-emerald-400 font-medium">
      {injectNotice}
    </div>
  {/if}

  <!-- Toolbar: Search + Work Group Filter -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 border-b border-neutral-200 dark:border-neutral-800 pb-3">
    <!-- Work Group Filter Dropdown -->
    <div class="flex items-center gap-2 text-xs">
      <span class="text-neutral-500 font-medium shrink-0">Work Group:</span>
      <select
        bind:value={filterGroup}
        onchange={() => (currentPage = 1)}
        class="px-3 py-1.5 bg-white dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 rounded-xl text-neutral-900 dark:text-white font-medium focus:outline-none focus:border-sky-500 shadow-2xs"
      >
        <option value="all">Semua Grup ({snippets.length})</option>
        {#each groups as g}
          <option value={g.name}>{g.name}</option>
        {/each}
      </select>
    </div>

    <!-- Search Box -->
    <div class="relative w-full sm:w-64">
      <input
        type="text"
        bind:value={searchQuery}
        oninput={() => (currentPage = 1)}
        placeholder="Cari snippet atau perintah..."
        class="w-full pl-8 pr-3 py-1.5 text-xs rounded-xl bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 text-neutral-900 dark:text-white placeholder-neutral-400 focus:outline-none focus:border-sky-500 shadow-2xs"
      />
      <svg class="w-3.5 h-3.5 text-neutral-400 absolute left-2.5 top-2.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
      </svg>
    </div>
  </div>

  <!-- Snippets Grid -->
  <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
    {#each paginatedSnippets as snip (snip.id)}
      <div class="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-2xl overflow-hidden group shadow-xs flex flex-col justify-between">
        <div>
          <div class="p-4 border-b border-neutral-100 dark:border-neutral-800/80">
            <div class="flex justify-between items-start gap-2">
              <h3 class="font-bold text-neutral-900 dark:text-white text-sm">{snip.label}</h3>
              <div class="flex items-center gap-1 shrink-0">
                <button
                  type="button"
                  onclick={() => injectSnippet(snip.command)}
                  disabled={!hasActiveSession()}
                  class="text-sky-600 dark:text-sky-400 hover:text-white hover:bg-sky-600 px-2 py-1 bg-sky-500/10 disabled:opacity-40 disabled:cursor-not-allowed rounded-lg text-xs font-semibold transition-colors"
                >
                  {t('snippets.inject')}
                </button>
                <button
                  type="button"
                  onclick={() => openEditModal(snip)}
                  class="text-neutral-500 hover:text-neutral-900 dark:hover:text-white px-2 py-1 bg-neutral-100 dark:bg-neutral-800 rounded-lg text-xs transition-colors"
                >
                  {t('common.edit')}
                </button>
                <button
                  type="button"
                  onclick={() => removeSnippet(snip.id, snip.label)}
                  class="text-rose-600 hover:text-white hover:bg-rose-600 px-2 py-1 bg-rose-500/10 rounded-lg text-xs transition-colors"
                >
                  {t('common.delete')}
                </button>
              </div>
            </div>
            {#if snip.description}
              <p class="text-neutral-500 text-xs mt-1">{snip.description}</p>
            {/if}
          </div>

          <div class="bg-neutral-50 dark:bg-neutral-950 p-3 font-mono text-xs text-neutral-800 dark:text-neutral-200 relative overflow-x-auto">
            <span class="opacity-50 select-none mr-2 text-sky-500">$</span>{snip.command}
          </div>
        </div>

        <!-- Tags & Group Footer -->
        <div class="px-4 py-2 bg-neutral-50/50 dark:bg-neutral-900/50 flex gap-1.5 flex-wrap border-t border-neutral-100 dark:border-neutral-800/80">
          {#each snip.tags as tag}
            <span class="px-2 py-0.5 bg-neutral-100 dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700/80 text-neutral-600 dark:text-neutral-300 rounded-md text-[11px] font-medium">
              #{tag}
            </span>
          {/each}
        </div>
      </div>
    {/each}

    {#if filteredSnippets.length === 0}
      <div class="md:col-span-2 py-16 px-4 border border-dashed border-neutral-300 dark:border-neutral-800 rounded-2xl text-center space-y-2">
        <h3 class="text-base font-bold text-neutral-900 dark:text-white">{t('snippets.emptyTitle')}</h3>
        <p class="text-xs text-neutral-500">{t('snippets.emptyBody')}</p>
      </div>
    {/if}
  </div>

  <!-- Pagination Component -->
  <Pagination
    totalItems={filteredSnippets.length}
    bind:page={currentPage}
    bind:pageSize
    pageSizeOptions={[10, 20, 50]}
  />
</div>

<!-- Modal Create / Edit Snippet -->
{#if isAddModalOpen}
  <div class="fixed inset-0 bg-black/60 dark:bg-black/80 flex items-center justify-center p-4 z-50 backdrop-blur-xs">
    <div
      role="dialog"
      aria-modal="true"
      class="bg-white dark:bg-[#141414] border border-neutral-200 dark:border-neutral-800 rounded-2xl p-6 max-w-lg w-full space-y-4 shadow-2xl animate-in fade-in zoom-in-95 duration-150 text-xs"
    >
      <div class="flex justify-between items-center border-b border-neutral-200 dark:border-neutral-800 pb-3">
        <h3 class="text-base font-bold text-neutral-900 dark:text-white">{editingId ? t('snippets.editSnippet') : t('snippets.addSnippet')}</h3>
        <span class="text-xs font-semibold bg-neutral-100 dark:bg-neutral-800 text-neutral-600 dark:text-neutral-400 px-2 py-0.5 rounded-md">
          {t('common.step', { current: creationStep, total: 2 })}
        </span>
      </div>

      {#if creationStep === 1}
        <div class="space-y-4">
          <div>
            <label class="block font-medium text-neutral-700 dark:text-neutral-300 mb-1.5">{t('snippets.bashCommand')} *</label>
            <textarea
              bind:value={newSnippet.cmd}
              rows="5"
              placeholder={t('snippets.commandPlaceholder')}
              class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-900 font-mono border border-neutral-300 dark:border-neutral-800 rounded-xl text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500"
            ></textarea>
          </div>
          <div class="flex justify-end gap-2 pt-2 border-t border-neutral-200 dark:border-neutral-800">
            <button
              type="button"
              onclick={resetModal}
              class="px-4 py-2 bg-neutral-100 dark:bg-neutral-800 hover:bg-neutral-200 dark:hover:bg-neutral-700 text-neutral-700 dark:text-neutral-300 rounded-lg font-semibold"
            >
              {t('common.cancel')}
            </button>
            <button
              type="button"
              onclick={() => (creationStep = 2)}
              disabled={!newSnippet.cmd.trim()}
              class="px-4 py-2 bg-sky-600 hover:bg-sky-500 disabled:opacity-50 text-white rounded-lg font-semibold shadow-md shadow-sky-600/20"
            >
              {t('common.next')} →
            </button>
          </div>
        </div>
      {:else}
        <form onsubmit={handleSaveSnippet} class="space-y-3">
          <div>
            <label class="block font-medium text-neutral-700 dark:text-neutral-300 mb-1">{t('snippets.label')} *</label>
            <input bind:value={newSnippet.label} required placeholder={t('snippets.labelPlaceholder')} class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 rounded-lg text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500" />
          </div>

          <div>
            <label class="block font-medium text-neutral-700 dark:text-neutral-300 mb-1">Work Group (Opsional)</label>
            <select
              bind:value={newSnippet.selectedGroup}
              class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 rounded-lg text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500"
            >
              <option value="">Tanpa Work Group</option>
              {#each snippetGroups as g}
                <option value={g.name}>{g.name}</option>
              {/each}
            </select>
          </div>

          <div>
            <label class="block font-medium text-neutral-700 dark:text-neutral-300 mb-1">{t('snippets.descriptionOptional')}</label>
            <input bind:value={newSnippet.desc} placeholder={t('snippets.descriptionPlaceholder')} class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 rounded-lg text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500" />
          </div>

          <div>
            <label class="block font-medium text-neutral-700 dark:text-neutral-300 mb-1">{t('snippets.tagsCommaSeparated')}</label>
            <input bind:value={newSnippet.tags} placeholder="misal: docker, logs, maintenance" class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 rounded-lg text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500" />
          </div>

          <div class="flex justify-between items-center pt-3 border-t border-neutral-200 dark:border-neutral-800">
            <button
              type="button"
              onclick={() => (creationStep = 1)}
              class="text-neutral-500 hover:text-neutral-900 dark:hover:text-white font-semibold transition-colors"
            >
              ← {t('common.back')}
            </button>
            <div class="flex gap-2">
              <button
                type="button"
                onclick={resetModal}
                class="px-4 py-2 bg-neutral-100 dark:bg-neutral-800 hover:bg-neutral-200 dark:hover:bg-neutral-700 text-neutral-700 dark:text-neutral-300 rounded-lg font-semibold"
              >
                {t('common.cancel')}
              </button>
              <button
                type="submit"
                class="px-4 py-2 bg-emerald-600 hover:bg-emerald-500 text-white rounded-lg font-semibold shadow-md shadow-emerald-600/20"
              >
                {t('snippets.saveSnippet')}
              </button>
            </div>
          </div>
        </form>
      {/if}
    </div>
  </div>
{/if}
