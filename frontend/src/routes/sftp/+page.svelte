<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import {
    listRemoteDir,
    readRemoteFile,
    writeRemoteFile,
    mkdirRemoteDir,
    deleteRemoteFile,
    renameRemoteFile,
    copyRemoteFile,
    type SftpFileEntry
  } from '$lib/api/sftp';
  import { listHosts, type HostRecord } from '$lib/api/hosts';

  let hosts = $state<HostRecord[]>([]);
  let currentHostId = $state('');
  let currentPath = $state('/');
  let files = $state<SftpFileEntry[]>([]);
  let isLoading = $state(false);
  let errorMsg = $state('');
  let successMsg = $state('');

  // Multi-selection state
  let selectedPaths = $state<Set<string>>(new Set());

  // Selected file for single operations
  let selectedFile = $state<SftpFileEntry | null>(null);

  // Clipboard for copy / cut
  let clipboard = $state<{
    action: 'copy' | 'cut';
    file: SftpFileEntry;
    sourceHostId: string;
  } | null>(null);

  // Modals state
  let showNewFileModal = $state(false);
  let newFileName = $state('');

  let showNewFolderModal = $state(false);
  let newFolderName = $state('');

  let showRenameModal = $state(false);
  let renameNewName = $state('');

  let showDeleteModal = $state(false);
  let fileToDelete = $state<SftpFileEntry | null>(null);
  let isBatchDeleting = $state(false);

  let showEditorModal = $state(false);
  let editorFilePath = $state('');
  let editorFileName = $state('');
  let editorContent = $state('');
  let isSavingEditor = $state(false);
  let isReadingEditor = $state(false);

  let showDiffModal = $state(false);
  let diffFileA = $state<SftpFileEntry | null>(null);
  let diffFileB = $state<SftpFileEntry | null>(null);
  let diffContentA = $state<string[]>([]);
  let diffContentB = $state<string[]>([]);
  let isLoadingDiff = $state(false);

  let uploadInputRef: HTMLInputElement;

  function toggleSelect(path: string) {
    const next = new Set(selectedPaths);
    if (next.has(path)) {
      next.delete(path);
    } else {
      next.add(path);
    }
    selectedPaths = next;
  }

  function toggleSelectAll() {
    if (selectedPaths.size === files.length && files.length > 0) {
      selectedPaths = new Set();
    } else {
      selectedPaths = new Set(files.map(f => f.path));
    }
  }

  async function handleCreateFile() {
    if (!newFileName.trim()) return;
    const dest = joinPath(currentPath, newFileName.trim());
    try {
      await writeRemoteFile(currentHostId, dest, []);
      notifySuccess(`Created file "${newFileName.trim()}"`);
      showNewFileModal = false;
      newFileName = '';
      await fetchFiles();
    } catch (e: any) {
      errorMsg = String(e?.message || e || 'Failed to create file');
    }
  }

  async function handleCreateFolder() {
    if (!newFolderName.trim()) return;
    const dest = joinPath(currentPath, newFolderName.trim());
    try {
      await mkdirRemoteDir(currentHostId, dest);
      notifySuccess(`Created folder "${newFolderName.trim()}"`);
      showNewFolderModal = false;
      newFolderName = '';
      await fetchFiles();
    } catch (e: any) {
      errorMsg = String(e?.message || e || 'Failed to create folder');
    }
  }

  async function handleBatchDelete() {
    if (selectedPaths.size === 0) return;
    isLoading = true;
    errorMsg = '';
    try {
      for (const path of selectedPaths) {
        await deleteRemoteFile(currentHostId, path);
      }
      notifySuccess(`Deleted ${selectedPaths.size} item(s)`);
      selectedPaths = new Set();
      await fetchFiles();
    } catch (e: any) {
      errorMsg = String(e?.message || e || 'Failed to delete some items');
    } finally {
      isLoading = false;
    }
  }

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
    selectedFile = null;
    try {
      files = await listRemoteDir(currentHostId, currentPath);
    } catch (e: any) {
      errorMsg = String(e?.message || e || 'Failed to list directory');
      files = [];
    } finally {
      isLoading = false;
    }
  }

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
      await fetchFiles();
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

  function joinPath(dir: string, name: string): string {
    if (dir === '/' || dir === '.' || !dir) return `/${name}`;
    return `${dir.replace(/\/+$/, '')}/${name}`;
  }

  function notifySuccess(msg: string) {
    successMsg = msg;
    setTimeout(() => {
      if (successMsg === msg) successMsg = '';
    }, 4000);
  }

  // --- RENAME ACTION ---
  function openRenameModal(file: SftpFileEntry) {
    selectedFile = file;
    renameNewName = file.name;
    showRenameModal = true;
  }

  async function submitRename() {
    if (!selectedFile || !renameNewName.trim() || renameNewName === selectedFile.name) {
      showRenameModal = false;
      return;
    }
    const oldPath = selectedFile.path;
    const parentDir = oldPath.substring(0, oldPath.lastIndexOf('/')) || '/';
    const newPath = joinPath(parentDir, renameNewName.trim());

    try {
      await renameRemoteFile(currentHostId, oldPath, newPath);
      notifySuccess(`Renamed "${selectedFile.name}" to "${renameNewName.trim()}"`);
      showRenameModal = false;
      await fetchFiles();
    } catch (e: any) {
      errorMsg = String(e?.message || e || 'Failed to rename');
    }
  }

  // --- DELETE ACTION ---
  function openDeleteModal(file: SftpFileEntry) {
    fileToDelete = file;
    showDeleteModal = true;
  }

  async function confirmDelete() {
    if (!fileToDelete) return;
    try {
      await deleteRemoteFile(currentHostId, fileToDelete.path);
      notifySuccess(`Deleted "${fileToDelete.name}"`);
      showDeleteModal = false;
      fileToDelete = null;
      await fetchFiles();
    } catch (e: any) {
      errorMsg = String(e?.message || e || 'Failed to delete file');
    }
  }

  // --- COPY / CUT / PASTE ---
  function triggerCopy(file: SftpFileEntry) {
    clipboard = { action: 'copy', file, sourceHostId: currentHostId };
    notifySuccess(`Copied "${file.name}" to clipboard`);
  }

  function triggerCut(file: SftpFileEntry) {
    clipboard = { action: 'cut', file, sourceHostId: currentHostId };
    notifySuccess(`Cut "${file.name}" to clipboard`);
  }

  async function triggerPaste() {
    if (!clipboard) return;
    const destPath = joinPath(currentPath, clipboard.file.name);
    if (clipboard.sourceHostId !== currentHostId) {
      errorMsg = 'Cross-host copy/cut is not supported yet.';
      return;
    }
    if (clipboard.file.path === destPath) {
      errorMsg = 'Source and destination paths are identical.';
      return;
    }

    try {
      if (clipboard.action === 'cut') {
        await renameRemoteFile(currentHostId, clipboard.file.path, destPath);
        notifySuccess(`Moved "${clipboard.file.name}" to current directory`);
        clipboard = null;
      } else {
        await copyRemoteFile(currentHostId, clipboard.file.path, destPath);
        notifySuccess(`Pasted "${clipboard.file.name}" to current directory`);
      }
      await fetchFiles();
    } catch (e: any) {
      errorMsg = String(e?.message || e || 'Failed to paste file');
    }
  }

  // --- OPEN TEXT EDITOR ---
  async function openTextEditor(file: SftpFileEntry) {
    if (file.is_dir) return;
    editorFileName = file.name;
    editorFilePath = file.path;
    isReadingEditor = true;
    showEditorModal = true;
    editorContent = '';
    errorMsg = '';

    try {
      const bytes = await readRemoteFile(currentHostId, file.path);
      const decoder = new TextDecoder('utf-8');
      editorContent = decoder.decode(new Uint8Array(bytes));
    } catch (e: any) {
      errorMsg = String(e?.message || e || 'Failed to read file content');
      showEditorModal = false;
    } finally {
      isReadingEditor = false;
    }
  }

  async function saveEditorContent() {
    if (!editorFilePath) return;
    isSavingEditor = true;
    try {
      const encoder = new TextEncoder();
      const bytes = Array.from(encoder.encode(editorContent));
      await writeRemoteFile(currentHostId, editorFilePath, bytes);
      notifySuccess(`Saved "${editorFileName}" successfully`);
      showEditorModal = false;
      await fetchFiles();
    } catch (e: any) {
      errorMsg = String(e?.message || e || 'Failed to save file');
    } finally {
      isSavingEditor = false;
    }
  }

  // --- UPLOAD FILES ---
  function openUploadDialog() {
    if (uploadInputRef) {
      uploadInputRef.value = '';
      uploadInputRef.click();
    }
  }

  async function handleFileUpload(event: Event) {
    const target = event.target as HTMLInputElement;
    const uploadFiles = target.files;
    if (!uploadFiles || uploadFiles.length === 0) return;

    isLoading = true;
    errorMsg = '';
    try {
      for (let i = 0; i < uploadFiles.length; i++) {
        const file = uploadFiles[i];
        const destPath = joinPath(currentPath, file.name);
        const arrayBuffer = await file.arrayBuffer();
        const uint8 = new Uint8Array(arrayBuffer);
        await writeRemoteFile(currentHostId, destPath, Array.from(uint8));
      }
      notifySuccess(`Successfully uploaded ${uploadFiles.length} file(s)`);
      await fetchFiles();
    } catch (e: any) {
      errorMsg = String(e?.message || e || 'Failed to upload file(s)');
    } finally {
      isLoading = false;
    }
  }

  // --- FILE COMPARISON (DIFF) ---
  function openDiffSelect(file: SftpFileEntry) {
    diffFileA = file;
    diffFileB = null;
    diffContentA = [];
    diffContentB = [];
    showDiffModal = true;
  }

  async function loadDiffFiles() {
    if (!diffFileA || !diffFileB) return;
    isLoadingDiff = true;
    try {
      const [bytesA, bytesB] = await Promise.all([
        readRemoteFile(currentHostId, diffFileA.path),
        readRemoteFile(currentHostId, diffFileB.path)
      ]);
      const decoder = new TextDecoder('utf-8');
      diffContentA = decoder.decode(new Uint8Array(bytesA)).split('\n');
      diffContentB = decoder.decode(new Uint8Array(bytesB)).split('\n');
    } catch (e: any) {
      errorMsg = String(e?.message || e || 'Failed to load files for diff');
    } finally {
      isLoadingDiff = false;
    }
  }
</script>

<div class="max-w-6xl mx-auto space-y-4">
  <!-- Hidden File Input for Upload -->
  <input
    type="file"
    multiple
    bind:this={uploadInputRef}
    onchange={handleFileUpload}
    class="hidden"
  />

  <!-- Header -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 mb-2">
    <div class="flex items-center gap-3">
      <div class="p-2.5 bg-emerald-500/10 text-emerald-500 dark:text-emerald-400 rounded-xl border border-emerald-500/20 shadow-sm">
        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"></path>
        </svg>
      </div>
      <div>
        <h1 class="text-2xl font-bold tracking-tight text-neutral-900 dark:text-white">SFTP File Manager</h1>
        <p class="text-xs text-neutral-500 dark:text-neutral-400">Manage remote files, inspect contents, compare diffs, and transfer assets over SSH.</p>
      </div>
    </div>

    <div class="flex items-center gap-2">
      {#if currentHostId}
        <a
          href="/session?host={currentHostId}"
          class="px-3 py-1.5 bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 hover:border-neutral-300 dark:hover:border-neutral-700 rounded-lg text-xs font-medium text-neutral-700 dark:text-neutral-300 hover:text-neutral-900 dark:hover:text-white flex items-center gap-1.5 transition-colors shadow-sm"
        >
          <svg class="w-3.5 h-3.5 text-sky-500 dark:text-sky-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"></path>
          </svg>
          <span>Open Terminal</span>
        </a>
      {/if}
      <button
        onclick={() => (showNewFileModal = true)}
        disabled={!currentHostId || isLoading}
        class="px-3 py-1.5 bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 hover:bg-neutral-50 dark:hover:bg-neutral-800 disabled:opacity-50 text-neutral-800 dark:text-neutral-200 rounded-lg text-xs font-semibold flex items-center gap-1.5 shadow-sm transition-colors"
        title="Create New File"
      >
        <svg class="w-4 h-4 text-sky-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 13h6m-3-3v6m5 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"></path>
        </svg>
        <span>New File</span>
      </button>
      <button
        onclick={() => (showNewFolderModal = true)}
        disabled={!currentHostId || isLoading}
        class="px-3 py-1.5 bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 hover:bg-neutral-50 dark:hover:bg-neutral-800 disabled:opacity-50 text-neutral-800 dark:text-neutral-200 rounded-lg text-xs font-semibold flex items-center gap-1.5 shadow-sm transition-colors"
        title="Create New Folder"
      >
        <svg class="w-4 h-4 text-amber-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 13h6m-3-3v6m-9 1V7a2 2 0 012-2h6l2 2h6a2 2 0 012 2v8a2 2 0 01-2 2H5a2 2 0 01-2-2z"></path>
        </svg>
        <span>New Folder</span>
      </button>
      <button
        onclick={openUploadDialog}
        disabled={!currentHostId || isLoading}
        class="px-3.5 py-1.5 bg-sky-600 hover:bg-sky-500 disabled:opacity-50 text-white rounded-lg text-xs font-semibold flex items-center gap-1.5 shadow-sm transition-colors"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12"></path>
        </svg>
        <span>Upload Files</span>
      </button>
    </div>
  </div>

  <!-- Host & Path Bar -->
  <div class="grid grid-cols-1 md:grid-cols-12 gap-2">
    <div class="md:col-span-4">
      <select
        bind:value={currentHostId}
        onchange={fetchFiles}
        class="w-full bg-white dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 rounded-lg px-3 py-2 text-sm text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500 transition-colors shadow-sm"
      >
        {#if hosts.length === 0}
          <option value="">No hosts available</option>
        {:else}
          {#each hosts as h}
            <option value={h.id}>{h.label || h.address} ({h.username}@{h.address})</option>
          {/each}
        {/if}
      </select>
    </div>

    <div class="md:col-span-8 flex gap-2">
      <button
        onclick={goUp}
        disabled={currentPath === '/' || !currentHostId}
        class="px-3 py-2 bg-white dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 rounded-lg text-neutral-600 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white hover:border-neutral-400 dark:hover:border-neutral-700 disabled:opacity-50 transition-colors shadow-sm"
        title="Go up one folder"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 10l7-7m0 0l7 7m-7-7v18"></path>
        </svg>
      </button>
      <input
        type="text"
        bind:value={currentPath}
        onkeydown={(e) => e.key === 'Enter' && fetchFiles()}
        placeholder="/"
        class="flex-1 bg-white dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 rounded-lg px-3 py-2 text-sm text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500 transition-colors font-mono shadow-sm"
      />
      <button
        onclick={fetchFiles}
        disabled={!currentHostId || isLoading}
        class="px-4 py-2 bg-neutral-800 hover:bg-neutral-700 dark:bg-neutral-800 dark:hover:bg-neutral-700 text-white rounded-lg text-sm font-medium transition-colors shadow-sm disabled:opacity-50"
      >
        Go
      </button>
      {#if clipboard}
        <button
          onclick={triggerPaste}
          class="px-3.5 py-2 bg-amber-500 hover:bg-amber-600 text-white rounded-lg text-xs font-semibold flex items-center gap-1.5 shadow-sm transition-colors animate-pulse"
          title="Paste '{clipboard.file.name}' ({clipboard.action})"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2"></path>
          </svg>
          <span>Paste ({clipboard.action})</span>
        </button>
      {/if}
    </div>
  </div>

  <!-- Selection & Multi-Action Toolbar -->
  <div class="flex flex-wrap items-center justify-between gap-2 p-2.5 bg-neutral-50 dark:bg-neutral-950/80 border border-neutral-200 dark:border-neutral-800 rounded-lg">
    <div class="flex items-center gap-2">
      <button
        onclick={toggleSelectAll}
        disabled={files.length === 0}
        class="px-2.5 py-1 text-xs font-semibold rounded border border-neutral-300 dark:border-neutral-700 bg-white dark:bg-neutral-900 text-neutral-700 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors shadow-sm disabled:opacity-50"
      >
        {selectedPaths.size === files.length && files.length > 0 ? 'Deselect All' : 'Select All'}
      </button>
      {#if selectedPaths.size > 0}
        <span class="text-xs font-medium text-sky-600 dark:text-sky-400">
          {selectedPaths.size} selected
        </span>
        <button
          onclick={handleBatchDelete}
          disabled={isLoading}
          class="px-2.5 py-1 bg-rose-600 hover:bg-rose-500 text-white rounded text-xs font-semibold flex items-center gap-1 shadow-sm transition-colors"
          title="Delete all selected items"
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"></path>
          </svg>
          <span>Batch Delete</span>
        </button>
      {/if}
    </div>

    <div class="flex items-center gap-2">
      {#if selectedFile && !selectedFile.is_dir}
        <button
          onclick={() => openTextEditor(selectedFile!)}
          class="px-2.5 py-1 text-xs font-semibold rounded border border-sky-300 dark:border-sky-800 bg-sky-50 dark:bg-sky-950/40 text-sky-700 dark:text-sky-300 hover:bg-sky-100 dark:hover:bg-sky-900/50 flex items-center gap-1 shadow-sm transition-colors"
          title="Edit Text"
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"></path>
          </svg>
          <span>Open Text</span>
        </button>
        <button
          onclick={() => openDiffSelect(selectedFile!)}
          class="px-2.5 py-1 text-xs font-semibold rounded border border-indigo-300 dark:border-indigo-800 bg-indigo-50 dark:bg-indigo-950/40 text-indigo-700 dark:text-indigo-300 hover:bg-indigo-100 dark:hover:bg-indigo-900/50 flex items-center gap-1 shadow-sm transition-colors"
          title="Compare with another file"
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4"></path>
          </svg>
          <span>Compare Files</span>
        </button>
      {/if}

      {#if selectedFile}
        <button
          onclick={() => triggerCopy(selectedFile!)}
          class="px-2.5 py-1 text-xs font-semibold rounded border border-neutral-300 dark:border-neutral-700 bg-white dark:bg-neutral-900 text-neutral-700 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800 flex items-center gap-1 shadow-sm transition-colors"
          title="Copy"
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"></path>
          </svg>
          <span>Copy</span>
        </button>
        <button
          onclick={() => triggerCut(selectedFile!)}
          class="px-2.5 py-1 text-xs font-semibold rounded border border-neutral-300 dark:border-neutral-700 bg-white dark:bg-neutral-900 text-neutral-700 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800 flex items-center gap-1 shadow-sm transition-colors"
          title="Cut"
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.121 14.121L19 19m-7-7l7-7m-7 7l-2.879 2.879a3 3 0 11-4.242-4.242 3 3 0 014.242 0L12 12zm0 0l-2.879-2.879a3 3 0 10-4.242 4.242 3 3 0 004.242 0L12 12z"></path>
          </svg>
          <span>Cut</span>
        </button>
      {/if}

      {#if clipboard}
        <button
          onclick={triggerPaste}
          class="px-2.5 py-1 bg-amber-500 hover:bg-amber-600 text-white rounded text-xs font-semibold flex items-center gap-1 shadow-sm transition-colors animate-pulse"
          title="Paste '{clipboard.file.name}'"
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2"></path>
          </svg>
          <span>Paste ({clipboard.action})</span>
        </button>
      {/if}
    </div>
  </div>

  <!-- Messages -->
  {#if errorMsg}
    <div class="p-3 bg-red-500/10 border border-red-500/20 rounded-lg text-red-600 dark:text-red-400 text-sm flex items-center justify-between">
      <span>{errorMsg}</span>
      <button
        onclick={() => (errorMsg = '')}
        class="text-xs font-semibold px-2 py-0.5 rounded hover:bg-red-500/20"
      >
        Dismiss
      </button>
    </div>
  {/if}
  {#if successMsg}
    <div class="p-3 bg-emerald-500/10 border border-emerald-500/20 rounded-lg text-emerald-600 dark:text-emerald-400 text-sm flex items-center justify-between">
      <span>{successMsg}</span>
      <button
        onclick={() => (successMsg = '')}
        class="text-xs font-semibold px-2 py-0.5 rounded hover:bg-emerald-500/20"
      >
        Dismiss
      </button>
    </div>
  {/if}

  <!-- Files Table -->
  <div class="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-xl overflow-hidden shadow-sm">
    <div class="overflow-x-auto">
      <table class="w-full text-left text-sm text-neutral-600 dark:text-neutral-400">
        <thead class="bg-neutral-50 dark:bg-neutral-950/60 text-xs uppercase text-neutral-500 border-b border-neutral-200 dark:border-neutral-800">
          <tr>
            <th class="px-3 py-3 w-10 text-center">
              <input
                type="checkbox"
                checked={selectedPaths.size === files.length && files.length > 0}
                onchange={toggleSelectAll}
                class="rounded border-neutral-300 dark:border-neutral-700 text-sky-600 focus:ring-0"
              />
            </th>
            <th class="px-4 py-3 font-semibold">Name</th>
            <th class="px-4 py-3 font-semibold w-24">Size</th>
            <th class="px-4 py-3 font-semibold w-36">Modified</th>
            <th class="px-4 py-3 font-semibold w-48 text-right">Actions</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-neutral-100 dark:divide-neutral-800/60">
          {#if isLoading}
            <tr>
              <td colspan="5" class="px-4 py-12 text-center text-neutral-500">
                <div class="inline-flex items-center gap-2">
                  <div class="w-4 h-4 border-2 border-sky-500 border-t-transparent rounded-full animate-spin"></div>
                  <span>Loading directory contents...</span>
                </div>
              </td>
            </tr>
          {:else if files.length === 0}
            <tr>
              <td colspan="5" class="px-4 py-12 text-center text-neutral-500">
                This directory is empty.
              </td>
            </tr>
          {:else}
            {#each files as file}
              <tr
                class="hover:bg-neutral-50 dark:hover:bg-neutral-800/50 transition-colors group {selectedFile?.path === file.path ? 'bg-sky-50/50 dark:bg-sky-950/20' : ''}"
                onclick={() => (selectedFile = file)}
              >
                <td class="px-3 py-2.5 text-center" onclick={(e) => e.stopPropagation()}>
                  <input
                    type="checkbox"
                    checked={selectedPaths.has(file.path)}
                    onchange={() => toggleSelect(file.path)}
                    class="rounded border-neutral-300 dark:border-neutral-700 text-sky-600 focus:ring-0"
                  />
                </td>
                <td class="px-4 py-2.5 flex items-center gap-3">
                  {#if file.is_dir}
                    <button
                      onclick={() => navigateTo(file.path)}
                      class="flex items-center gap-2.5 text-left group-hover:text-sky-600 dark:group-hover:text-sky-400 transition-colors"
                    >
                      <svg class="w-5 h-5 text-amber-500 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
                        <path d="M2 6a2 2 0 012-2h5l2 2h5a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z"></path>
                      </svg>
                      <span class="font-medium text-neutral-900 dark:text-neutral-100">{file.name}</span>
                    </button>
                  {:else}
                    <div class="flex items-center gap-2.5 truncate">
                      <svg class="w-5 h-5 text-neutral-400 dark:text-neutral-500 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z"></path>
                      </svg>
                      <span class="text-neutral-800 dark:text-neutral-200 truncate">{file.name}</span>
                    </div>
                  {/if}
                </td>
                <td class="px-4 py-2.5 whitespace-nowrap text-xs text-neutral-500 font-mono">
                  {file.is_dir ? '-' : formatSize(file.size)}
                </td>
                <td class="px-4 py-2.5 whitespace-nowrap text-xs text-neutral-500">
                  {file.mtime ? new Date(file.mtime * 1000).toLocaleString() : '-'}
                </td>
                <td class="px-4 py-2.5 whitespace-nowrap text-right">
                  <div class="flex items-center justify-end gap-1 opacity-90 group-hover:opacity-100">
                    {#if !file.is_dir}
                      <button
                        onclick={(e) => { e.stopPropagation(); openTextEditor(file); }}
                        class="p-1.5 text-neutral-500 hover:text-sky-600 dark:hover:text-sky-400 hover:bg-neutral-100 dark:hover:bg-neutral-800 rounded transition-colors"
                        title="Edit Text"
                      >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"></path>
                        </svg>
                      </button>
                      <button
                        onclick={(e) => { e.stopPropagation(); openDiffSelect(file); }}
                        class="p-1.5 text-neutral-500 hover:text-indigo-600 dark:hover:text-indigo-400 hover:bg-neutral-100 dark:hover:bg-neutral-800 rounded transition-colors"
                        title="Compare with another file"
                      >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4"></path>
                        </svg>
                      </button>
                      <button
                        onclick={(e) => { e.stopPropagation(); triggerCopy(file); }}
                        class="p-1.5 text-neutral-500 hover:text-neutral-900 dark:hover:text-white hover:bg-neutral-100 dark:hover:bg-neutral-800 rounded transition-colors"
                        title="Copy"
                      >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"></path>
                        </svg>
                      </button>
                    {/if}
                    <button
                      onclick={(e) => { e.stopPropagation(); triggerCut(file); }}
                      class="p-1.5 text-neutral-500 hover:text-amber-600 dark:hover:text-amber-400 hover:bg-neutral-100 dark:hover:bg-neutral-800 rounded transition-colors"
                      title="Cut (Move)"
                    >
                      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.121 14.121L19 19m-7-7l7-7m-7 7l-2.879 2.879a3 3 0 11-4.242-4.242 3 3 0 014.242 0L12 12zm0 0l-2.879-2.879a3 3 0 10-4.242 4.242 3 3 0 004.242 0L12 12z"></path>
                      </svg>
                    </button>
                    <button
                      onclick={(e) => { e.stopPropagation(); openRenameModal(file); }}
                      class="p-1.5 text-neutral-500 hover:text-neutral-900 dark:hover:text-white hover:bg-neutral-100 dark:hover:bg-neutral-800 rounded transition-colors"
                      title="Rename"
                    >
                      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z"></path>
                      </svg>
                    </button>
                    <button
                      onclick={(e) => { e.stopPropagation(); openDeleteModal(file); }}
                      class="p-1.5 text-neutral-500 hover:text-red-600 dark:hover:text-red-400 hover:bg-neutral-100 dark:hover:bg-neutral-800 rounded transition-colors"
                      title="Delete"
                    >
                      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"></path>
                      </svg>
                    </button>
                  </div>
                </td>
              </tr>
            {/each}
          {/if}
        </tbody>
      </table>
    </div>
  </div>
</div>

<!-- NEW FILE MODAL -->
{#if showNewFileModal}
  <div class="fixed inset-0 z-50 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4">
    <div class="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-xl p-5 max-w-md w-full shadow-2xl space-y-4">
      <h3 class="text-base font-semibold text-neutral-900 dark:text-white">Create New File</h3>
      <div>
        <label class="block text-xs font-medium text-neutral-500 dark:text-neutral-400 mb-1" for="new-file-name">File Name</label>
        <input
          id="new-file-name"
          type="text"
          bind:value={newFileName}
          placeholder="config.json or script.sh"
          onkeydown={(e) => e.key === 'Enter' && handleCreateFile()}
          class="w-full bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded-lg px-3 py-2 text-sm text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500 font-mono"
        />
      </div>
      <div class="flex justify-end gap-2 pt-2">
        <button
          onclick={() => { showNewFileModal = false; newFileName = ''; }}
          class="px-3 py-1.5 border border-neutral-300 dark:border-neutral-700 hover:bg-neutral-100 dark:hover:bg-neutral-800 text-neutral-700 dark:text-neutral-300 text-xs font-semibold rounded-lg"
        >
          Cancel
        </button>
        <button
          onclick={handleCreateFile}
          disabled={!newFileName.trim()}
          class="px-3 py-1.5 bg-sky-600 hover:bg-sky-500 disabled:opacity-50 text-white text-xs font-semibold rounded-lg"
        >
          Create File
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- NEW FOLDER MODAL -->
{#if showNewFolderModal}
  <div class="fixed inset-0 z-50 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4">
    <div class="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-xl p-5 max-w-md w-full shadow-2xl space-y-4">
      <h3 class="text-base font-semibold text-neutral-900 dark:text-white">Create New Folder</h3>
      <div>
        <label class="block text-xs font-medium text-neutral-500 dark:text-neutral-400 mb-1" for="new-folder-name">Folder Name</label>
        <input
          id="new-folder-name"
          type="text"
          bind:value={newFolderName}
          placeholder="my-project or logs"
          onkeydown={(e) => e.key === 'Enter' && handleCreateFolder()}
          class="w-full bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded-lg px-3 py-2 text-sm text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500 font-mono"
        />
      </div>
      <div class="flex justify-end gap-2 pt-2">
        <button
          onclick={() => { showNewFolderModal = false; newFolderName = ''; }}
          class="px-3 py-1.5 border border-neutral-300 dark:border-neutral-700 hover:bg-neutral-100 dark:hover:bg-neutral-800 text-neutral-700 dark:text-neutral-300 text-xs font-semibold rounded-lg"
        >
          Cancel
        </button>
        <button
          onclick={handleCreateFolder}
          disabled={!newFolderName.trim()}
          class="px-3 py-1.5 bg-amber-600 hover:bg-amber-500 disabled:opacity-50 text-white text-xs font-semibold rounded-lg"
        >
          Create Folder
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- RENAME MODAL -->
{#if showRenameModal}
  <div class="fixed inset-0 z-50 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4">
    <div class="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-xl p-5 max-w-md w-full shadow-2xl space-y-4">
      <h3 class="text-base font-semibold text-neutral-900 dark:text-white">Rename File or Folder</h3>
      <div>
        <label class="block text-xs font-medium text-neutral-500 dark:text-neutral-400 mb-1" for="new-name-input">New Name</label>
        <input
          id="new-name-input"
          type="text"
          bind:value={renameNewName}
          onkeydown={(e) => e.key === 'Enter' && submitRename()}
          class="w-full bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded-lg px-3 py-2 text-sm text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500"
        />
      </div>
      <div class="flex justify-end gap-2 pt-2">
        <button
          onclick={() => (showRenameModal = false)}
          class="px-3 py-1.5 border border-neutral-300 dark:border-neutral-700 hover:bg-neutral-100 dark:hover:bg-neutral-800 text-neutral-700 dark:text-neutral-300 text-xs font-semibold rounded-lg"
        >
          Cancel
        </button>
        <button
          onclick={submitRename}
          class="px-3 py-1.5 bg-sky-600 hover:bg-sky-500 text-white text-xs font-semibold rounded-lg"
        >
          Rename
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- DELETE MODAL -->
{#if showDeleteModal && fileToDelete}
  <div class="fixed inset-0 z-50 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4">
    <div class="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-xl p-5 max-w-md w-full shadow-2xl space-y-4">
      <h3 class="text-base font-semibold text-neutral-900 dark:text-white">Confirm Delete</h3>
      <p class="text-xs text-neutral-600 dark:text-neutral-400">
        Are you sure you want to permanently delete <strong class="text-neutral-900 dark:text-white font-mono">{fileToDelete.name}</strong>?
        This action cannot be undone.
      </p>
      <div class="flex justify-end gap-2 pt-2">
        <button
          onclick={() => (showDeleteModal = false)}
          class="px-3 py-1.5 border border-neutral-300 dark:border-neutral-700 hover:bg-neutral-100 dark:hover:bg-neutral-800 text-neutral-700 dark:text-neutral-300 text-xs font-semibold rounded-lg"
        >
          Cancel
        </button>
        <button
          onclick={confirmDelete}
          class="px-3 py-1.5 bg-red-600 hover:bg-red-500 text-white text-xs font-semibold rounded-lg"
        >
          Delete
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- TEXT EDITOR MODAL -->
{#if showEditorModal}
  <div class="fixed inset-0 z-50 bg-black/75 backdrop-blur-sm flex items-center justify-center p-4">
    <div class="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-xl w-full max-w-4xl max-h-[90vh] flex flex-col shadow-2xl overflow-hidden">
      <div class="px-4 py-3 border-b border-neutral-200 dark:border-neutral-800 flex items-center justify-between bg-neutral-50 dark:bg-neutral-950/70">
        <div class="flex items-center gap-2">
          <svg class="w-4 h-4 text-sky-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"></path>
          </svg>
          <span class="text-sm font-semibold text-neutral-900 dark:text-white font-mono">{editorFileName}</span>
          <span class="text-xs text-neutral-500 font-mono truncate max-w-sm">({editorFilePath})</span>
        </div>
        <div class="flex items-center gap-2">
          <button
            onclick={() => (showEditorModal = false)}
            class="px-3 py-1 border border-neutral-300 dark:border-neutral-700 hover:bg-neutral-100 dark:hover:bg-neutral-800 text-neutral-700 dark:text-neutral-300 text-xs font-semibold rounded-md"
          >
            Close
          </button>
          <button
            onclick={saveEditorContent}
            disabled={isSavingEditor || isReadingEditor}
            class="px-3.5 py-1 bg-sky-600 hover:bg-sky-500 disabled:opacity-50 text-white text-xs font-semibold rounded-md flex items-center gap-1.5 shadow-sm"
          >
            {#if isSavingEditor}
              <div class="w-3 h-3 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
            {/if}
            <span>Save</span>
          </button>
        </div>
      </div>

      <div class="flex-1 p-3 bg-neutral-950 flex flex-col min-h-[350px]">
        {#if isReadingEditor}
          <div class="flex-1 flex items-center justify-center text-neutral-400 text-xs gap-2">
            <div class="w-4 h-4 border-2 border-sky-500 border-t-transparent rounded-full animate-spin"></div>
            <span>Fetching remote file...</span>
          </div>
        {:else}
          <textarea
            bind:value={editorContent}
            spellcheck="false"
            class="w-full flex-1 bg-transparent text-neutral-200 font-mono text-xs leading-relaxed focus:outline-none resize-none p-2 border border-neutral-800 rounded"
          ></textarea>
        {/if}
      </div>
    </div>
  </div>
{/if}

<!-- FILE DIFF / COMPARISON MODAL -->
{#if showDiffModal && diffFileA}
  <div class="fixed inset-0 z-50 bg-black/75 backdrop-blur-sm flex items-center justify-center p-4">
    <div class="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-xl w-full max-w-5xl max-h-[90vh] flex flex-col shadow-2xl overflow-hidden">
      <div class="px-4 py-3 border-b border-neutral-200 dark:border-neutral-800 flex items-center justify-between bg-neutral-50 dark:bg-neutral-950/70">
        <div>
          <h3 class="text-sm font-semibold text-neutral-900 dark:text-white">Compare Files</h3>
          <p class="text-xs text-neutral-500">Side-by-side text diff comparison</p>
        </div>
        <button
          onclick={() => (showDiffModal = false)}
          class="px-3 py-1 border border-neutral-300 dark:border-neutral-700 hover:bg-neutral-100 dark:hover:bg-neutral-800 text-neutral-700 dark:text-neutral-300 text-xs font-semibold rounded-md"
        >
          Close
        </button>
      </div>

      <div class="p-3 border-b border-neutral-200 dark:border-neutral-800 bg-neutral-100/50 dark:bg-neutral-900/50 flex flex-wrap items-center gap-3">
        <div class="text-xs">
          <span class="text-neutral-500">File A:</span>
          <span class="font-mono font-semibold text-sky-600 dark:text-sky-400 ml-1">{diffFileA.name}</span>
        </div>
        <div class="text-xs flex items-center gap-2 flex-1">
          <span class="text-neutral-500">File B:</span>
          <select
            bind:value={diffFileB}
            onchange={loadDiffFiles}
            class="bg-white dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded px-2 py-1 text-xs text-neutral-900 dark:text-white focus:outline-none"
          >
            <option value={null}>Select comparison file...</option>
            {#each files.filter((f) => !f.is_dir && f.path !== diffFileA?.path) as f}
              <option value={f}>{f.name}</option>
            {/each}
          </select>
        </div>
      </div>

      <div class="flex-1 overflow-auto p-3 bg-neutral-950 font-mono text-xs">
        {#if isLoadingDiff}
          <div class="py-16 text-center text-neutral-400 flex items-center justify-center gap-2">
            <div class="w-4 h-4 border-2 border-sky-500 border-t-transparent rounded-full animate-spin"></div>
            <span>Reading files for comparison...</span>
          </div>
        {:else if !diffFileB}
          <div class="py-16 text-center text-neutral-500">
            Please select File B from the dropdown above to view differences.
          </div>
        {:else}
          <div class="grid grid-cols-2 gap-4">
            <div class="space-y-0.5">
              <div class="text-sky-400 font-semibold mb-2 pb-1 border-b border-neutral-800">{diffFileA.name}</div>
              {#each diffContentA as line, idx}
                {@const isDiff = diffContentB[idx] !== line}
                <div class="px-1 py-0.5 whitespace-pre-wrap break-all rounded {isDiff ? 'bg-red-950/40 text-red-200 border-l-2 border-red-500' : 'text-neutral-400'}">
                  <span class="text-neutral-600 select-none mr-2">{idx + 1}</span>{line || ' '}
                </div>
              {/each}
            </div>

            <div class="space-y-0.5">
              <div class="text-emerald-400 font-semibold mb-2 pb-1 border-b border-neutral-800">{diffFileB.name}</div>
              {#each diffContentB as line, idx}
                {@const isDiff = diffContentA[idx] !== line}
                <div class="px-1 py-0.5 whitespace-pre-wrap break-all rounded {isDiff ? 'bg-emerald-950/40 text-emerald-200 border-l-2 border-emerald-500' : 'text-neutral-400'}">
                  <span class="text-neutral-600 select-none mr-2">{idx + 1}</span>{line || ' '}
                </div>
              {/each}
            </div>
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}
