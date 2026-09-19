<script lang="ts">
  import { onMount } from 'svelte';
  import { listRemoteDir, type SftpFileEntry } from '$lib/api/sftp';
  import { listHosts, type HostRecord } from '$lib/api/hosts';

  let hosts: HostRecord[] = [];
  let currentHostId = '';
  let currentPath = '/';
  let files: SftpFileEntry[] = [];
  let isLoading = false;
  let errorMsg = '';

  async function loadHosts() {
    try {
      hosts = await listHosts();
      if (hosts.length > 0 && !currentHostId) {
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
      errorMsg = String(e);
      files = [];
    } finally {
      isLoading = false;
    }
  }

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
  <div class="flex items-center gap-3 mb-6">
    <div class="p-2 bg-emerald-500/10 text-emerald-400 rounded-lg border border-emerald-500/20">
      <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"></path></svg>
    </div>
    <div>
      <h1 class="text-2xl font-bold text-white">File Manager (SFTP)</h1>
      <p class="text-sm text-neutral-400">Browse and manage remote files over active SSH sessions.</p>
    </div>
  </div>

  <div class="flex gap-2">
    <select bind:value={currentHostId} on:change={fetchFiles} class="bg-neutral-900 border border-neutral-800 rounded-lg px-3 py-2 text-sm text-white focus:outline-none focus:border-sky-500 transition-colors w-64">
      {#if hosts.length === 0}
        <option value="">No hosts available</option>
      {:else}
        {#each hosts as h}
          <option value={h.id}>{h.label || h.address}</option>
        {/each}
      {/if}
    </select>
    
    <div class="flex-1 flex gap-2">
      <button on:click={goUp} disabled={currentPath === '/'} class="px-3 py-2 bg-neutral-900 border border-neutral-800 rounded-lg text-neutral-400 hover:text-white hover:border-neutral-700 disabled:opacity-50 transition-colors">
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 10l7-7m0 0l7 7m-7-7v18"></path></svg>
      </button>
      <input type="text" bind:value={currentPath} on:keydown={e => e.key === 'Enter' && fetchFiles()} class="flex-1 bg-neutral-900 border border-neutral-800 rounded-lg px-3 py-2 text-sm text-white focus:outline-none focus:border-sky-500 transition-colors font-mono" />
      <button on:click={fetchFiles} class="px-4 py-2 bg-sky-600 hover:bg-sky-500 text-white rounded-lg text-sm font-medium transition-colors">
        Go
      </button>
    </div>
  </div>

  {#if errorMsg}
    <div class="p-4 bg-red-500/10 border border-red-500/20 rounded-lg text-red-400 text-sm">
      {errorMsg}
    </div>
  {/if}

  <div class="bg-neutral-900 border border-neutral-800 rounded-xl overflow-hidden shadow-xl">
    <table class="w-full text-left text-sm text-neutral-400">
      <thead class="bg-neutral-950/50 text-xs uppercase text-neutral-500 border-b border-neutral-800">
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
            <tr class="border-b border-neutral-800/50 hover:bg-neutral-800/50 transition-colors group cursor-pointer" on:click={() => file.is_dir && navigateTo(file.path)}>
              <td class="px-4 py-2.5 flex items-center gap-3">
                {#if file.is_dir}
                  <svg class="w-5 h-5 text-amber-500" fill="currentColor" viewBox="0 0 20 20"><path d="M2 6a2 2 0 012-2h5l2 2h5a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z"></path></svg>
                {:else}
                  <svg class="w-5 h-5 text-neutral-500" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z"></path></svg>
                {/if}
                <span class="text-neutral-200 group-hover:text-white transition-colors truncate">{file.name}</span>
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