<script lang="ts">
  import { onMount } from 'svelte';
  import { injectIntoActiveSession, hasActiveSession } from '$lib/stores/activeSession.svelte';
  import { listSnippets, saveSnippet as saveSnippetApi, deleteSnippet as deleteSnippetApi, type SnippetRecord } from '$lib/api/snippets';
  import { t } from '$lib/i18n/index.svelte';
  import PageHeader from '$lib/components/PageHeader.svelte';

  let isAddModalOpen = $state(false);
  let creationStep = $state(1); // 1: command, 2: details
  let injectNotice = $state('');
  let backendAvailable = $state(true);
  let editingId = $state<string | null>(null);

  let snippets = $state<SnippetRecord[]>([]);

  let newSnippet = $state({
    cmd: "",
    label: "",
    desc: "",
    tags: ""
  });

  onMount(async () => {
    try {
      snippets = await listSnippets();
    } catch {
      backendAvailable = false;
    }
  });

  function resetModal() {
    editingId = null;
    newSnippet = { cmd: "", label: "", desc: "", tags: "" };
    creationStep = 1;
    isAddModalOpen = false;
  }

  function openEditModal(snip: SnippetRecord) {
    editingId = snip.id;
    newSnippet = { cmd: snip.command, label: snip.label, desc: snip.description, tags: snip.tags.join(', ') };
    creationStep = 2;
    isAddModalOpen = true;
  }

  function nextStep() {
    if (newSnippet.cmd.trim()) creationStep = 2;
  }

  async function saveSnippet(e: Event) {
    e.preventDefault();
    if (!newSnippet.label) return;

    const input = {
      id: editingId ?? undefined,
      label: newSnippet.label,
      description: newSnippet.desc,
      command: newSnippet.cmd,
      tags: newSnippet.tags.split(',').map(t => t.trim()).filter(Boolean)
    };

    try {
      const saved = await saveSnippetApi(input);
      snippets = editingId
        ? snippets.map((s) => (s.id === saved.id ? saved : s))
        : [...snippets, saved];
    } catch {
      backendAvailable = false;
    }

    resetModal();
  }

  async function removeSnippet(id: string) {
    try {
      await deleteSnippetApi(id);
      snippets = snippets.filter((s) => s.id !== id);
    } catch {
      backendAvailable = false;
    }
  }

  function injectSnippet(cmd: string) {
    const injected = injectIntoActiveSession(cmd);
    injectNotice = injected ? t('snippets.injectedNotice') : t('snippets.noActiveSessionNotice');
    setTimeout(() => (injectNotice = ''), 3000);
  }
</script>

<div class="max-w-5xl mx-auto space-y-6">
  <PageHeader
    icon={['M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4']}
    accent="sky"
    title={t('snippets.title')}
    subtitle={t('snippets.subtitle')}
  >
    {#snippet actions()}
      <button
        onclick={() => { resetModal(); isAddModalOpen = true; }}
        class="px-4 py-2 bg-sky-600 hover:bg-sky-500 text-white font-medium text-sm rounded-lg transition-colors flex items-center gap-2 shadow shadow-sky-600/20">
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"></path></svg>
        {t('snippets.addSnippet')}
      </button>
    {/snippet}
  </PageHeader>
  {#if !backendAvailable}
    <p class="text-amber-500 text-xs -mt-4">{t('common.backendUnavailable')}</p>
  {/if}

  {#if injectNotice}
    <div class="px-4 py-2 bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded text-xs text-sky-600 dark:text-sky-400 shadow-sm dark:shadow-none">{injectNotice}</div>
  {/if}

  <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
    {#each snippets as snip}
      <div class="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-lg overflow-hidden group shadow-sm dark:shadow-none">
        <div class="p-4 border-b border-neutral-100 dark:border-neutral-800">
          <div class="flex justify-between items-start">
            <h3 class="font-semibold text-neutral-900 dark:text-white">{snip.label}</h3>
            <div class="flex items-center gap-1">
              <button
                onclick={() => injectSnippet(snip.command)}
                disabled={!hasActiveSession()}
                class="text-sky-600 dark:text-sky-400 hover:text-sky-500 dark:hover:text-sky-300 px-2 py-1 bg-sky-500/10 hover:bg-sky-500/20 disabled:opacity-40 disabled:cursor-not-allowed rounded text-xs transition-colors">{t('snippets.inject')}</button>
              <button
                onclick={() => openEditModal(snip)}
                class="text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white px-2 py-1 bg-neutral-100 dark:bg-neutral-800 hover:bg-neutral-200 dark:hover:bg-neutral-700 rounded text-xs transition-colors">{t('common.edit')}</button>
              <button
                onclick={() => removeSnippet(snip.id)}
                class="text-red-600 dark:text-red-400 hover:text-red-500 dark:hover:text-red-300 px-2 py-1 bg-red-500/10 hover:bg-red-500/20 rounded text-xs transition-colors">{t('common.delete')}</button>
            </div>
          </div>
          <p class="text-neutral-500 dark:text-neutral-400 text-xs mt-1">{snip.description}</p>
        </div>
        <div class="bg-neutral-50 dark:bg-neutral-950 p-3 font-mono text-sm text-neutral-700 dark:text-neutral-300 relative">
          <span class="opacity-50 select-none mr-2">$</span>{snip.command}
        </div>
        <div class="px-4 py-2 bg-neutral-50 dark:bg-neutral-900 flex gap-1 border-t border-neutral-100 dark:border-neutral-800">
          {#each snip.tags as tag}
            <span class="px-2 py-0.5 bg-neutral-100 dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 text-neutral-600 dark:text-neutral-400 rounded text-xs">{tag}</span>
          {/each}
        </div>
      </div>
    {/each}
    {#if snippets.length === 0}
      <div class="md:col-span-2 p-10 border border-dashed border-neutral-300 dark:border-neutral-800 rounded-xl text-center">
        <h3 class="text-lg font-medium text-neutral-900 dark:text-white mb-1">{t('snippets.emptyTitle')}</h3>
        <p class="text-sm text-neutral-500 dark:text-neutral-400">{t('snippets.emptyBody')}</p>
      </div>
    {/if}
  </div>
</div>

<!-- Create Snippet Modal (2 Steps) -->
{#if isAddModalOpen}
  <div class="fixed inset-0 bg-black/50 dark:bg-black/70 flex items-center justify-center p-4 z-50">
    <div class="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-lg p-6 max-w-lg w-full space-y-4 shadow-2xl">
      <div class="flex justify-between items-center border-b border-neutral-200 dark:border-neutral-800 pb-3">
        <h3 class="text-xl font-bold text-neutral-900 dark:text-white">{editingId ? t('snippets.editSnippet') : t('snippets.addSnippet')}</h3>
        <span class="text-xs text-neutral-500 dark:text-neutral-500 font-medium bg-neutral-100 dark:bg-neutral-800 px-2 py-1 rounded">{t('common.step', { current: creationStep, total: 2 })}</span>
      </div>

      {#if creationStep === 1}
        <div class="space-y-4">
          <div>
            <label for="snip-cmd" class="block text-xs font-medium text-neutral-500 dark:text-neutral-400 uppercase mb-2">{t('snippets.bashCommand')}</label>
            <textarea
              id="snip-cmd"
              bind:value={newSnippet.cmd}
              rows="4"
              placeholder={t('snippets.commandPlaceholder')}
              class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-950 font-mono border border-neutral-300 dark:border-neutral-800 rounded text-sm text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500"></textarea>
          </div>
          <div class="flex justify-end gap-2 pt-2 border-t border-neutral-200 dark:border-neutral-800">
            <button
              onclick={resetModal}
              class="px-4 py-2 bg-neutral-100 dark:bg-neutral-800 hover:bg-neutral-200 dark:hover:bg-neutral-700 text-neutral-700 dark:text-neutral-300 rounded text-sm font-medium">{t('common.cancel')}</button>
            <button
              onclick={nextStep}
              disabled={!newSnippet.cmd.trim()}
              class="px-4 py-2 bg-sky-600 hover:bg-sky-500 disabled:opacity-50 text-white rounded text-sm font-medium">{t('common.next')}</button>
          </div>
        </div>
      {:else}
        <form onsubmit={saveSnippet} class="space-y-4">
          <div>
            <label for="snip-label" class="block text-xs font-medium text-neutral-500 dark:text-neutral-400 uppercase mb-1">{t('snippets.label')}</label>
            <input id="snip-label" bind:value={newSnippet.label} required placeholder={t('snippets.labelPlaceholder')} class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded text-sm text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500" />
          </div>

          <div>
            <label for="snip-desc" class="block text-xs font-medium text-neutral-500 dark:text-neutral-400 uppercase mb-1">{t('snippets.descriptionOptional')}</label>
            <input id="snip-desc" bind:value={newSnippet.desc} placeholder={t('snippets.descriptionPlaceholder')} class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded text-sm text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500" />
          </div>

          <div>
            <label for="snip-tags" class="block text-xs font-medium text-neutral-500 dark:text-neutral-400 uppercase mb-1">{t('snippets.tagsCommaSeparated')}</label>
            <input id="snip-tags" bind:value={newSnippet.tags} placeholder={t('snippets.tagsPlaceholder')} class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded text-sm text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500" />
          </div>

          <div class="flex items-center justify-between p-3 bg-neutral-50 dark:bg-neutral-950 border border-neutral-200 dark:border-neutral-800 rounded opacity-50 hidden">
            <div>
              <span class="block text-sm text-neutral-700 dark:text-neutral-300">{t('snippets.sharePublicly')}</span>
              <span class="block text-xs text-neutral-500">{t('common.comingSoon')}</span>
            </div>
            <button
              type="button"
              disabled
              role="switch"
              aria-checked="false"
              aria-label={`${t('snippets.sharePublicly')} (${t('common.comingSoon')})`}
              class="w-10 h-6 rounded-full bg-neutral-200 dark:bg-neutral-800 border border-neutral-300 dark:border-neutral-700 cursor-not-allowed relative">
              <span class="absolute top-0.5 left-0.5 w-4 h-4 rounded-full bg-neutral-400 dark:bg-neutral-500"></span>
            </button>
          </div>

          <div class="flex justify-between items-center pt-2 border-t border-neutral-200 dark:border-neutral-800">
            <button
              type="button"
              onclick={() => creationStep = 1}
              class="text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white text-sm font-medium transition-colors">← {t('common.back')}</button>
            <div class="flex gap-2">
              <button
                type="button"
                onclick={resetModal}
                class="px-4 py-2 bg-neutral-100 dark:bg-neutral-800 hover:bg-neutral-200 dark:hover:bg-neutral-700 text-neutral-700 dark:text-neutral-300 rounded text-sm font-medium">{t('common.cancel')}</button>
              <button
                type="submit"
                class="px-4 py-2 bg-emerald-600 hover:bg-emerald-500 text-white rounded text-sm font-medium">{t('snippets.saveSnippet')}</button>
            </div>
          </div>
        </form>
      {/if}
    </div>
  </div>
{/if}
