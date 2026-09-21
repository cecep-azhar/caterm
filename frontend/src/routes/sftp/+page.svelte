<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import { listRemoteDir, type SftpFileEntry } from '$lib/api/sftp';
  import { listHosts, type HostRecord } from '$lib/api/hosts';

  let hosts = $state<HostRecord[]>([]);
  let currentHostId = $state('');
  let currentPath = $state('/');
  let files = $state<SftpFileEntry[]>([]);
  let isLoading = $state(false);
  let errorMsg = $state('');

  async function loadHosts() {
    try {
      hosts = await listHosts();
      const hostParam = page.url.searchParams.get('host');
      if (hostParam && hosts.some((h) => h.id === hostParam)) {
        currentHostId = hostParam;
      } else if (hosts.length > 0 && !currentHostId) {
        currentHostId = hosts[0].id;
      }
    } catch (e: any) {
      errorMsg = String(e);
    }
  }

  async function fetchFiles() {
    if (!currentHostId) return;
    isLoading = true;
    errorMsg = '';
    try {
      files = await listRemoteDir(currentHostId, currentPath);
    } catch (e: any) {
      errorMsg = String(e?.message || e || 'Failed to list directory');
      files = [];
    } finally {
      isLoading = false;
    }
  }

  // Handle URL query parameter changes
  $effect(() => {
    const hostParam = page.url.searchParams.get('host');
    if (hostParam && hostParam !== currentHostId && hosts.some((h) => h.id === hostParam)) {
      currentHostId = hostParam;
      fetchFiles();
    }
  });

  onMount(async () => {
    await loadHosts();
    if (currentHostId) {
      fetchFiles();
    }
  });

  function navigateTo(path: string) {
    currentPath = path;
    fetchFiles();
  }

  function goUp() {
    if (currentPath === '/') return;
    const parts = currentPath.split('/').filter(Boolean);
    parts.pop();
    currentPath = '/' + parts.join('/');
    if (currentPath === '') currentPath = '/';
    fetchFiles();
  }

  function formatSize(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }
</script>

<div class="max-w-4xl mx-auto space-y-4">
  <div class="flex items-center justify-between mb-6">
    <div class="flex items-center gap-3">
      <div class="p-2 bg-emerald-500/10 text-emerald-500 dark:text-emerald-400 rounded-lg border border-emerald-500/20">
        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"></path></svg>
      </div>
      <div>
        <h1 class="text-2xl font-bold text-neutral-900 dark:text-white">File Manager (SFTP)</h1>
        <p class="text-sm text-neutral-500 dark:text-neutral-400">Browse and manage remote files over active SSH sessions.</p>
      </div>
    </div>

    {#if currentHostId}
      <a
        href="/session?host={currentHostId}"
        class="px-3 py-1.5 bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 hover:border-neutral-300 dark:hover:border-neutral-700 rounded-lg text-xs font-medium text-neutral-700 dark:text-neutral-300 hover:text-neutral-900 dark:hover:text-white flex items-center gap-1.5 transition-colors shadow-sm dark:shadow-none"
      >
        <svg class="w-3.5 h-3.5 text-sky-500 dark:text-sky-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"></path>
        </svg>
        <span>Open Terminal Session</span>
      </a>
    {/if}
  </div>

  <div class="flex gap-2">
    <select
      bind:value={currentHostId}
      onchange={fetchFiles}
      class="bg-white dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 rounded-lg px-3 py-2 text-sm text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500 transition-colors w-64 shadow-sm dark:shadow-none"
    >
      {#if hosts.length === 0}
        <option value="">No hosts available</option>
      {:else}
        {#each hosts as h}
          <option value={h.id}>{h.label || h.address}</option>
        {/each}
      {/if}
    </select>
    
    <div class="flex-1 flex gap-2">
      <button
        onclick={goUp}
        disabled={currentPath === '/'}
        class="px-3 py-2 bg-white dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 rounded-lg text-neutral-600 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white hover:border-neutral-400 dark:hover:border-neutral-700 disabled:opacity-50 transition-colors shadow-sm dark:shadow-none"
        title="Go up one folder"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 10l7-7m0 0l7 7m-7-7v18"></path></svg>
      </button>
      <input
        type="text"
        bind:value={currentPath}
        onkeydown={(e) => e.key === 'Enter' && fetchFiles()}
        class="flex-1 bg-white dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 rounded-lg px-3 py-2 text-sm text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500 transition-colors font-mono shadow-sm dark:shadow-none"
      />
      <button
        onclick={fetchFiles}
        class="px-4 py-2 bg-sky-600 hover:bg-sky-500 text-white rounded-lg text-sm font-medium transition-colors shadow-sm"
      >
        Go
      </button>
    </div>
  </div>

  {#if errorMsg}
    <div class="p-4 bg-red-500/10 border border-red-500/20 rounded-lg text-red-500 dark:text-red-400 text-sm flex items-center justify-between">
      <span>{errorMsg}</span>
      <button
        onclick={fetchFiles}
        class="px-3 py-1 bg-red-500/20 hover:bg-red-500/30 text-red-600 dark:text-red-300 rounded text-xs font-medium transition-colors"
      >
        Retry
      </button>
    </div>
  {/if}

  <div class="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-xl overflow-hidden shadow-sm dark:shadow-xl">
    <table class="w-full text-left text-sm text-neutral-600 dark:text-neutral-400">
      <thead class="bg-neutral-50 dark:bg-neutral-950/50 text-xs uppercase text-neutral-500 border-b border-neutral-200 dark:border-neutral-800">
        <tr>
          <th class="px-4 py-3 font-medium">Name</th>
          <th class="px-4 py-3 font-medium w-24">Size</th>
          <th class="px-4 py-3 font-medium w-32">Modified</th>
        </tr>
      </thead>
      <tbody>
        {#if isLoading}
          <tr>
            <td colspan="3" class="px-4 py-8 text-center text-neutral-500">Loading directory contents...</td>
          </tr>
        {:else if files.length === 0}
          <tr>
            <td colspan="3" class="px-4 py-8 text-center text-neutral-500">Directory is empty</td>
          </tr>
        {:else}
          {#each files as file}
            <tr
              class="border-b border-neutral-100 dark:border-neutral-800/50 hover:bg-neutral-50 dark:hover:bg-neutral-800/50 transition-colors group cursor-pointer"
              onclick={() => file.is_dir && navigateTo(file.path)}
            >
              <td class="px-4 py-2.5 flex items-center gap-3">
                {#if file.is_dir}
                  <svg class="w-5 h-5 text-amber-500" fill="currentColor" viewBox="0 0 20 20"><path d="M2 6a2 2 0 012-2h5l2 2h5a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z"></path></svg>
                {:else}
                  <svg class="w-5 h-5 text-neutral-400 dark:text-neutral-500" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z"></path></svg>
                {/if}
                <span class="text-neutral-800 dark:text-neutral-200 group-hover:text-neutral-950 dark:group-hover:text-white transition-colors truncate">{file.name}</span>
              </td>
              <td class="px-4 py-2.5 whitespace-nowrap">
                {#if !file.is_dir}
                  {formatSize(file.size)}
                {/if}
              </td>
              <td class="px-4 py-2.5 whitespace-nowrap text-xs">
                {new Date(file.mtime * 1000).toLocaleDateString()}
              </td>
            </tr>
          {/each}
        {/if}
      </tbody>
    </table>
  </div>
</div>
