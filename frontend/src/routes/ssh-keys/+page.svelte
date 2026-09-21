<script lang="ts">
  import { onMount } from 'svelte';
  import { listKeys, generateKey, importKey, deleteKey, deployPublicKey, type KeyRecord } from '$lib/api/keys';
  import { listHosts, type HostRecord } from '$lib/api/hosts';

  let keys: KeyRecord[] = [];
  let hosts: HostRecord[] = [];
  let isLoading = true;
  let errorMsg = '';
  let successMsg = '';

  // Deploy modal
  let showDeployModal = false;
  let deployKeyId = '';
  let deployHostId = '';
  let isDeploying = false;

  // Generate modal
  let showGenerateModal = false;
  let generateName = '';
  let generateAlg = 'Ed25519';
  let isGenerating = false;

  // Import modal
  let showImportModal = false;
  let importName = '';
  let importPem = '';
  let importPassphrase = '';
  let showImportPassphrase = false;
  let isImporting = false;

  async function loadKeys() {
    isLoading = true;
    errorMsg = '';
    successMsg = '';
    try {
      [keys, hosts] = await Promise.all([listKeys(), listHosts()]);
    } catch (e: any) {
      errorMsg = String(e);
    } finally {
      isLoading = false;
    }
  }

  onMount(loadKeys);

  async function handleGenerate() {
    if (!generateName.trim()) return;
    isGenerating = true;
    errorMsg = '';
    successMsg = '';
    try {
      await generateKey({ name: generateName, algorithm: generateAlg });
      showGenerateModal = false;
      generateName = '';
      await loadKeys();
    } catch (e: any) {
      errorMsg = String(e);
    } finally {
      isGenerating = false;
    }
  }

  async function handleImport() {
    if (!importName.trim() || !importPem.trim()) return;
    isImporting = true;
    errorMsg = '';
    successMsg = '';
    try {
      await importKey(importName, importPem, importPassphrase || undefined);
      showImportModal = false;
      importName = '';
      importPem = '';
      importPassphrase = '';
      await loadKeys();
    } catch (e: any) {
      errorMsg = String(e);
    } finally {
      isImporting = false;
    }
  }

  async function handleDelete(id: string, name: string) {
    if (!confirm(`Delete key "${name}"?\n(Make sure it is not currently used by any Host)`)) return;
    errorMsg = '';
    successMsg = '';
    try {
      await deleteKey(id);
      await loadKeys();
    } catch (e: any) {
      errorMsg = String(e);
    }
  }

  function openDeployModal(keyId: string) {
    deployKeyId = keyId;
    deployHostId = hosts.length > 0 ? hosts[0].id : '';
    showDeployModal = true;
    errorMsg = '';
    successMsg = '';
  }

  async function handleDeploy() {
    if (!deployKeyId || !deployHostId) return;
    isDeploying = true;
    errorMsg = '';
    successMsg = '';
    try {
      await deployPublicKey(deployHostId, deployKeyId);
      showDeployModal = false;
      const targetHost = hosts.find(h => h.id === deployHostId);
      successMsg = `Public key deployed successfully to server ${targetHost?.label || deployHostId}!`;
    } catch (e: any) {
      errorMsg = String(e);
    } finally {
      isDeploying = false;
    }
  }
</script>

<div class="max-w-5xl mx-auto space-y-6">
  <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4 pb-4 border-b border-neutral-200 dark:border-neutral-800/80 mb-6">
    <div class="flex items-center gap-3">
      <div class="p-2 bg-rose-500/10 text-rose-400 rounded-lg border border-rose-500/20">
        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z"></path>
        </svg>
      </div>
      <div>
        <h1 class="text-2xl font-bold text-neutral-900 dark:text-white tracking-tight">SSH Keys</h1>
        <p class="text-sm text-neutral-500 dark:text-neutral-400 mt-1">Manage, generate, import, and deploy SSH keypairs (RSA / ED25519) securely stored in the local vault.</p>
      </div>
    </div>
    <div class="flex items-center gap-2">
      <button on:click={() => showImportModal = true} class="px-3.5 py-2 bg-neutral-100 hover:bg-neutral-200 dark:bg-neutral-800 dark:hover:bg-neutral-700 text-neutral-700 dark:text-neutral-200 rounded-lg text-sm font-medium transition-colors border border-neutral-200 dark:border-neutral-700">
        Import
      </button>
      <button on:click={() => showGenerateModal = true} class="px-4 py-2 bg-sky-600 hover:bg-sky-500 text-white rounded-lg text-sm font-medium transition-colors shadow shadow-sky-600/20">
        Generate
      </button>
    </div>
  </div>

  {#if errorMsg}
    <div class="p-4 bg-red-500/10 border border-red-500/20 rounded-lg text-red-400 text-sm">
      {errorMsg}
    </div>
  {/if}

  {#if successMsg}
    <div class="p-4 bg-emerald-500/10 border border-emerald-500/20 rounded-lg text-emerald-400 text-sm">
      {successMsg}
    </div>
  {/if}

  {#if isLoading}
    <div class="p-8 text-center text-neutral-500 text-sm">Loading keys...</div>
  {:else if keys.length === 0}
    <div class="p-8 border border-neutral-800 rounded-xl bg-neutral-900/30 text-center flex flex-col items-center justify-center">
      <svg class="w-12 h-12 text-neutral-600 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"></path>
      </svg>
      <h3 class="text-lg font-medium text-white mb-2">No keys in vault</h3>
      <p class="text-sm text-neutral-400 max-w-md">Generate a new ED25519 keypair or import an existing private key to authenticate securely without passwords.</p>
      <div class="mt-6 flex gap-3">
        <button on:click={() => showImportModal = true} class="px-4 py-2 bg-neutral-800 hover:bg-neutral-700 text-white rounded-lg text-sm font-medium transition-colors border border-neutral-700">
          Import Key
        </button>
        <button on:click={() => showGenerateModal = true} class="px-4 py-2 bg-sky-600 hover:bg-sky-500 text-white rounded-lg text-sm font-medium transition-colors shadow-lg shadow-sky-600/20">
          Generate Keypair
        </button>
      </div>
    </div>
  {:else}
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      {#each keys as key (key.id)}
        <div class="flex flex-col p-4 bg-neutral-900 border border-neutral-800 rounded-xl hover:border-neutral-700 transition-colors group">
          <div class="flex items-center justify-between mb-3">
            <h3 class="font-medium text-white flex items-center gap-2">
              <svg class="w-4 h-4 text-rose-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z"></path></svg>
              {key.name}
            </h3>
            <span class="text-xs px-2 py-0.5 rounded-full bg-neutral-800 text-neutral-400 font-mono">
              {key.algorithm}
            </span>
          </div>
          
          <div class="space-y-2 mb-4">
            <div class="bg-black/30 rounded p-2 text-xs font-mono text-neutral-500 break-all select-all">
              {key.fingerprint}
            </div>
          </div>
          
          <div class="mt-auto pt-3 border-t border-neutral-800/50 flex justify-between items-center opacity-40 group-hover:opacity-100 transition-opacity">
            <span class="text-xs text-neutral-500" title={key.createdAt}>
              {new Date(key.createdAt).toLocaleDateString()}
            </span>
            <div class="flex items-center gap-2">
              <button 
                on:click={() => openDeployModal(key.id)}
                class="text-xs text-sky-400 hover:text-sky-300 font-medium px-2 py-1 rounded hover:bg-sky-500/10 transition-colors"
              >
                Deploy to Server
              </button>
              <button 
                on:click={() => handleDelete(key.id, key.name)}
                class="text-xs text-rose-500 hover:text-rose-400 font-medium px-2 py-1 rounded hover:bg-rose-500/10 transition-colors"
              >
                Delete
              </button>
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

{#if showDeployModal}
<div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm">
  <div class="bg-neutral-900 border border-neutral-800 rounded-xl w-full max-w-md shadow-2xl p-6">
    <h3 class="text-lg font-semibold text-white mb-4">Deploy Public Key to Remote Server</h3>
    
    <div class="space-y-4">
      <div>
        <label class="block text-xs font-medium text-neutral-400 mb-1">Target Host</label>
        {#if hosts.length === 0}
          <div class="p-3 bg-neutral-950 border border-neutral-800 rounded-lg text-sm text-neutral-500">
            No hosts available. Add a host in the Hosts tab first.
          </div>
        {:else}
          <select bind:value={deployHostId} class="w-full bg-black/40 border border-neutral-800 rounded-lg px-3 py-2 text-sm text-white focus:outline-none focus:border-sky-500 transition-colors">
            {#each hosts as h}
              <option value={h.id}>{h.label} ({h.username}@{h.address})</option>
            {/each}
          </select>
        {/if}
      </div>
      <p class="text-xs text-neutral-500">
        CATerm will automatically connect using the host's current saved credentials and append this key to `~/.ssh/authorized_keys`.
      </p>
    </div>

    <div class="flex justify-end gap-3 mt-6">
      <button on:click={() => showDeployModal = false} class="px-4 py-2 text-neutral-400 hover:text-white transition-colors text-sm font-medium">Cancel</button>
      <button on:click={handleDeploy} disabled={!deployHostId || isDeploying} class="px-4 py-2 bg-sky-600 hover:bg-sky-500 disabled:opacity-50 text-white rounded-lg text-sm font-medium transition-colors">
        {isDeploying ? 'Deploying...' : 'Deploy Public Key'}
      </button>
    </div>
  </div>
</div>
{/if}

{#if showGenerateModal}
<div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm">
  <div class="bg-neutral-900 border border-neutral-800 rounded-xl w-full max-w-md shadow-2xl p-6">
    <h3 class="text-lg font-semibold text-white mb-4">Generate New SSH Key</h3>
    
    <div class="space-y-4">
      <div>
        <label class="block text-xs font-medium text-neutral-400 mb-1">Key Name</label>
        <input type="text" bind:value={generateName} placeholder="e.g. Personal Desktop, Production Admin" class="w-full bg-black/40 border border-neutral-800 rounded-lg px-3 py-2 text-sm text-white focus:outline-none focus:border-sky-500 transition-colors" />
      </div>
      <div>
        <label class="block text-xs font-medium text-neutral-400 mb-1">Algorithm</label>
        <select bind:value={generateAlg} class="w-full bg-black/40 border border-neutral-800 rounded-lg px-3 py-2 text-sm text-white focus:outline-none focus:border-sky-500 transition-colors">
          <option value="Ed25519">Ed25519 (Recommended, fast & secure)</option>
          <option value="RSA-4096">RSA-4096 (Legacy compatibility)</option>
        </select>
      </div>
      <p class="text-xs text-neutral-500">
        The private key will be generated locally and stored securely encrypted in your vault. It will never leave this device.
      </p>
    </div>

    <div class="flex justify-end gap-3 mt-6">
      <button on:click={() => showGenerateModal = false} class="px-4 py-2 text-neutral-400 hover:text-white transition-colors text-sm font-medium">Cancel</button>
      <button on:click={handleGenerate} disabled={!generateName.trim() || isGenerating} class="px-4 py-2 bg-sky-600 hover:bg-sky-500 disabled:opacity-50 text-white rounded-lg text-sm font-medium transition-colors">
        {isGenerating ? 'Generating...' : 'Generate Key'}
      </button>
    </div>
  </div>
</div>
{/if}

{#if showImportModal}
<div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm">
  <div class="bg-neutral-900 border border-neutral-800 rounded-xl w-full max-w-2xl shadow-2xl p-6">
    <h3 class="text-lg font-semibold text-white mb-4">Import SSH Private Key</h3>
    
    <div class="space-y-4">
      <div>
        <label class="block text-xs font-medium text-neutral-400 mb-1">Key Name</label>
        <input type="text" bind:value={importName} placeholder="e.g. AWS Legacy Key" class="w-full bg-black/40 border border-neutral-800 rounded-lg px-3 py-2 text-sm text-white focus:outline-none focus:border-sky-500 transition-colors" />
      </div>
      <div>
        <label class="block text-xs font-medium text-neutral-400 mb-1">Private Key (PEM Format)</label>
        <textarea bind:value={importPem} rows="6" placeholder="-----BEGIN OPENSSH PRIVATE KEY-----..." class="w-full font-mono bg-black/40 border border-neutral-800 rounded-lg px-3 py-2 text-sm text-white focus:outline-none focus:border-sky-500 transition-colors resize-none"></textarea>
      </div>
      <div>
        <label class="block text-xs font-medium text-neutral-400 mb-1">Passphrase (if key is encrypted)</label>
        <div class="relative">
          <input
            type={showImportPassphrase ? 'text' : 'password'}
            bind:value={importPassphrase}
            placeholder="Leave blank if not encrypted"
            class="w-full bg-black/40 border border-neutral-800 rounded-lg pl-3 pr-10 py-2 text-sm text-white focus:outline-none focus:border-sky-500 transition-colors"
          />
          <button
            type="button"
            on:click={() => (showImportPassphrase = !showImportPassphrase)}
            class="absolute right-2.5 top-1/2 -translate-y-1/2 text-neutral-400 hover:text-white transition-colors p-1"
            aria-label={showImportPassphrase ? 'Hide passphrase' : 'Show passphrase'}
            title={showImportPassphrase ? 'Hide passphrase' : 'Show passphrase'}
          >
            {#if showImportPassphrase}
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
      </div>
      <p class="text-xs text-neutral-500">
        Your key will be re-encrypted using the local vault's zero-knowledge master password.
      </p>
    </div>

    <div class="flex justify-end gap-3 mt-6">
      <button on:click={() => showImportModal = false} class="px-4 py-2 text-neutral-400 hover:text-white transition-colors text-sm font-medium">Cancel</button>
      <button on:click={handleImport} disabled={!importName.trim() || !importPem.trim() || isImporting} class="px-4 py-2 bg-sky-600 hover:bg-sky-500 disabled:opacity-50 text-white rounded-lg text-sm font-medium transition-colors">
        {isImporting ? 'Importing...' : 'Import Key'}
      </button>
    </div>
  </div>
</div>
{/if}