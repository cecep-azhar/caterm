<script lang="ts">
  import { validateVaultPassword, MIN_VAULT_PASSWORD_LEN } from '$lib/api/vault';
  import { exportEncryptedBackup, importEncryptedBackup } from '$lib/api/backup';
  import AiSettingsForm from '$lib/components/AiSettingsForm.svelte';
  import FeedbackWidget from '$lib/components/FeedbackWidget.svelte';
  import { showToast } from '$lib/stores/uiNotifications.svelte';

  let activeTab = $state('updates'); // 'updates' | 'ai' | 'subscription' | 'sync' | 'security' | 'backup' | 'shortcuts' | 'feedback'
  let vaultPassword = $state('');
  let showVaultPassword = $state(false);
  let vaultMessage = $state('');
  let vaultMessageKind = $state<'success' | 'error'>('success');

  let backupPassphrase = $state('');
  let showBackupPassphrase = $state(false);
  let backupMsg = $state('');
  let backupMsgKind = $state<'success' | 'error'>('success');
  let restorePassphrase = $state('');
  let showRestorePassphrase = $state(false);
  let restoreMsg = $state('');
  let restoreMsgKind = $state<'success' | 'error'>('success');

  async function handleExport(e: Event) {
    e.preventDefault();
    if (backupPassphrase.length < 8) {
      showToast('Passphrase must be at least 8 characters.', 'error');
      return;
    }

    try {
      showToast('Creating encrypted backup...', 'info');
      const b64 = await exportEncryptedBackup(backupPassphrase);
      
      const blob = new Blob([b64], { type: 'text/plain' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `caterm-backup-${new Date().toISOString().split('T')[0]}.catb`;
      a.click();
      URL.revokeObjectURL(url);

      showToast('Backup saved successfully!', 'success');
      backupPassphrase = '';
    } catch (err: any) {
      showToast(err?.message || 'Failed to export backup', 'error');
    }
  }

  async function handleImport(e: Event) {
    e.preventDefault();
    if (restorePassphrase.length < 8) {
      showToast('Passphrase must be at least 8 characters.', 'error');
      return;
    }

    try {
      const input = document.createElement('input');
      input.type = 'file';
      input.accept = '.catb,.enc,.txt';
      input.onchange = async () => {
        const file = input.files?.[0];
        if (!file) return;
        
        showToast('Decrypting and importing backup...', 'info');
        
        const reader = new FileReader();
        reader.onload = async (event) => {
          try {
            const b64 = event.target?.result as string;
            const importedCount = await importEncryptedBackup(b64, restorePassphrase);
            showToast(`Restore successful! Restored ${importedCount} items.`, 'success');
            restorePassphrase = '';
          } catch (err: any) {
            showToast(err?.message || 'Failed to restore backup', 'error');
          }
        };
        reader.readAsText(file);
      };
      input.click();
    } catch (err: any) {
      showToast(err?.message || 'Failed to restore backup', 'error');
    }
  }

  async function updateVaultPassword(e: Event) {
    e.preventDefault();
    try {
      await validateVaultPassword(vaultPassword);
      showToast('Master password verified and saved.', 'success');
      vaultPassword = '';
    } catch (err) {
      const msg =
        (err as { message?: string })?.message ??
        `Vault password must be at least ${MIN_VAULT_PASSWORD_LEN} characters.`;
      showToast(msg, 'error');
    }
  }
</script>

<div class="max-w-4xl mx-auto space-y-6">
  <div class="pb-4 border-b border-neutral-200 dark:border-neutral-800/80 mb-6">
    <h1 class="text-2xl font-bold text-neutral-900 dark:text-white tracking-tight">Settings</h1>
    <p class="text-neutral-500 dark:text-neutral-400 text-sm mt-1">Configure global preferences, system updates, and zero-knowledge local security.</p>
  </div>

  <!-- Settings Tabs -->
  <div class="border-b border-neutral-800 flex gap-4 overflow-x-auto">
    <button 
      onclick={() => activeTab = 'updates'} 
      class="pb-3 whitespace-nowrap text-sm font-medium transition-colors border-b-2 {activeTab === 'updates' ? 'border-sky-500 text-white' : 'border-transparent text-neutral-400 hover:text-neutral-200'}">
      Updates
    </button>
    <button
      onclick={() => activeTab = 'ai'}
      class="pb-3 whitespace-nowrap text-sm font-medium transition-colors border-b-2 {activeTab === 'ai' ? 'border-sky-500 text-white' : 'border-transparent text-neutral-400 hover:text-neutral-200'}">
      AI Assistant
    </button>
    <button
      onclick={() => activeTab = 'subscription'}
      class="pb-3 whitespace-nowrap text-sm font-medium transition-colors border-b-2 {activeTab === 'subscription' ? 'border-sky-500 text-white' : 'border-transparent text-neutral-400 hover:text-neutral-200'}">
      Subscription
    </button>
    <button
      onclick={() => activeTab = 'sync'}
      class="pb-3 whitespace-nowrap text-sm font-medium transition-colors border-b-2 {activeTab === 'sync' ? 'border-sky-500 text-white' : 'border-transparent text-neutral-400 hover:text-neutral-200'}">
      Cloud Sync E2EE
    </button>
    <button 
      onclick={() => activeTab = 'security'} 
      class="pb-3 whitespace-nowrap text-sm font-medium transition-colors border-b-2 {activeTab === 'security' ? 'border-sky-500 text-white' : 'border-transparent text-neutral-400 hover:text-neutral-200'}">
      Vault Security
    </button>
    <button 
      onclick={() => activeTab = 'backup'} 
      class="pb-3 whitespace-nowrap text-sm font-medium transition-colors border-b-2 {activeTab === 'backup' ? 'border-sky-500 text-white' : 'border-transparent text-neutral-400 hover:text-neutral-200'}">
      Backup & Restore
    </button>
    <button 
      onclick={() => activeTab = 'shortcuts'} 
      class="pb-3 whitespace-nowrap text-sm font-medium transition-colors border-b-2 {activeTab === 'shortcuts' ? 'border-sky-500 text-white' : 'border-transparent text-neutral-400 hover:text-neutral-200'}">
      Shortcuts
    </button>
    <button
      onclick={() => activeTab = 'feedback'}
      class="pb-3 whitespace-nowrap text-sm font-medium transition-colors border-b-2 {activeTab === 'feedback' ? 'border-sky-500 text-white' : 'border-transparent text-neutral-400 hover:text-neutral-200'}">
      Feedback
    </button>
  </div>

  {#if activeTab === 'updates'}
    <div class="bg-neutral-900 border border-neutral-800 rounded-lg p-6 space-y-4">
      <div class="flex justify-between items-center">
        <div>
          <h2 class="text-lg font-semibold text-white">Application Updates</h2>
          <p class="text-neutral-400 text-sm">Current installed version: <span class="font-mono text-sky-400">v2.1.7</span></p>
        </div>
        <button class="px-4 py-2 bg-sky-600 hover:bg-sky-500 text-white text-sm font-medium rounded-md transition-colors">
          Check for Updates
        </button>
      </div>
    </div>
  {:else if activeTab === 'ai'}
    <div class="bg-neutral-900 border border-neutral-800 rounded-lg p-6 space-y-4">
      <div>
        <h2 class="text-lg font-semibold text-white">AI Ops Assistant</h2>
        <p class="text-neutral-400 text-sm">
          Endpoint LLM, credential, dan model yang dipakai AI Chat dan Prompt Studio.
        </p>
      </div>
      <AiSettingsForm />
    </div>
  {:else if activeTab === 'subscription'}
    <div class="bg-neutral-900 border border-neutral-800 rounded-lg p-6 space-y-6">
      <div>
        <h2 class="text-lg font-semibold text-white">Subscription Plan</h2>
        <p class="text-neutral-400 text-sm mt-1">Manage your CATerm plan and usage limits.</p>
      </div>

      <div class="border border-neutral-800 rounded-lg p-5 bg-neutral-950 flex items-center justify-between">
        <div>
          <div class="flex items-center gap-2">
            <span class="font-semibold text-white">Free Plan</span>
            <span class="text-xs bg-neutral-800 text-neutral-300 border border-neutral-700 px-2 py-0.5 rounded">Current</span>
          </div>
          <p class="text-neutral-400 text-xs mt-2">Unlimited hosts &amp; snippets, 100% local-first, zero-knowledge encryption.</p>
        </div>
      </div>

      <div class="border border-sky-500/30 rounded-lg p-5 bg-neutral-950 flex flex-col gap-4 opacity-90 hidden">
        <div class="flex justify-between items-start">
          <div>
            <div class="flex items-center gap-2">
              <span class="font-semibold text-white">Pro Plan</span>
              <span class="text-xs bg-amber-500/10 text-amber-400 border border-amber-500/20 px-2 py-0.5 rounded">Next Feature / Disabled</span>
            </div>
            <p class="text-neutral-400 text-xs mt-2">Unlimited hosts &amp; snippets, AI Agent CATerm, priority support, and team collaboration.</p>
          </div>
          <div class="text-right shrink-0">
            <div class="text-2xl font-bold text-white">$1 <span class="text-sm font-normal text-neutral-500">/ mo</span></div>
            <p class="text-neutral-500 text-xs">then $3/mo, +$1/team member</p>
          </div>
        </div>
        <button disabled class="w-full py-2.5 bg-sky-600/30 text-sky-200/60 text-sm font-semibold rounded-md cursor-not-allowed border border-sky-500/20">
          Upgrade to Pro ($1/mo, $3 next, +$1/team)
        </button>
      </div>

      <div class="pt-4 border-t border-neutral-800/80 flex items-center justify-between">
        <div>
          <p class="text-xs text-neutral-300 font-medium">Support Independent Open Development</p>
          <p class="text-[11px] text-neutral-500">CATerm is free and local-first. Donations help maintain active development.</p>
        </div>
        <a
          href="https://paypal.me/cecepazhar"
          target="_blank"
          rel="noopener"
          class="inline-flex items-center gap-2 px-4 py-2 bg-[#0070ba] hover:bg-[#005ea6] text-white text-xs font-semibold rounded-lg transition-colors"
        >
          <span>Donate via PayPal</span>
        </a>
      </div>
    </div>
  {:else if activeTab === 'sync'}
    <div class="bg-neutral-900 border border-neutral-800 rounded-lg p-6 space-y-6">
      <div>
        <h2 class="text-lg font-semibold text-white">Cloud Sync E2EE (End-to-End Encrypted)</h2>
        <p class="text-neutral-400 text-sm mt-1">
          Zero-Knowledge remote backup and cross-device sync. Your vault key never leaves your local hardware.
        </p>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <!-- Personal Tier Placeholder -->
        <div class="border border-neutral-800 rounded-lg p-5 bg-neutral-950 flex flex-col justify-between opacity-75">
          <div>
            <div class="flex justify-between items-center">
              <span class="font-semibold text-white">Personal Cloud Sync</span>
              <span class="text-xs bg-amber-500/10 text-amber-400 border border-amber-500/20 px-2 py-0.5 rounded">Next Feature / Disabled</span>
            </div>
            <div class="text-2xl font-bold text-white mt-3">$1 <span class="text-sm font-normal text-neutral-500">/ mo</span></div>
            <p class="text-neutral-400 text-xs mt-2">Encrypted backup for up to 5 devices with local key derivation.</p>
          </div>
          <button disabled class="mt-4 w-full py-2 bg-neutral-800 text-neutral-500 text-xs font-medium rounded cursor-not-allowed">
            Coming Soon
          </button>
        </div>

        <!-- Team Tier Placeholder -->
        <div class="border border-neutral-800 rounded-lg p-5 bg-neutral-950 flex flex-col justify-between opacity-75">
          <div>
            <div class="flex justify-between items-center">
              <span class="font-semibold text-white">Team & Org Sync</span>
              <span class="text-xs bg-amber-500/10 text-amber-400 border border-amber-500/20 px-2 py-0.5 rounded">Next Feature / Disabled</span>
            </div>
            <div class="text-2xl font-bold text-white mt-3">$1 <span class="text-sm font-normal text-neutral-500">/ user / mo</span></div>
            <p class="text-neutral-400 text-xs mt-2">Shared team vaults, audit trails, and multi-user access control.</p>
          </div>
          <button disabled class="mt-4 w-full py-2 bg-neutral-800 text-neutral-500 text-xs font-medium rounded cursor-not-allowed">
            Coming Soon
          </button>
        </div>
      </div>
    </div>
  {:else if activeTab === 'security'}
    <div class="bg-neutral-900 border border-neutral-800 rounded-lg p-6 space-y-4">
      <h2 class="text-lg font-semibold text-white">Zero-Knowledge Vault Configuration</h2>
      <p class="text-neutral-400 text-sm">CATerm enforces local-first encryption for host records, passwords, and private keys.</p>

      <form onsubmit={updateVaultPassword} class="max-w-md space-y-4 pt-2">
        <div>
          <label for="vault-pass" class="block text-xs font-medium text-neutral-400 uppercase mb-1">Master Vault Password (min. {MIN_VAULT_PASSWORD_LEN} chars)</label>
          <div class="relative">
            <input
              id="vault-pass"
              type={showVaultPassword ? 'text' : 'password'}
              minlength={MIN_VAULT_PASSWORD_LEN}
              required
              bind:value={vaultPassword}
              placeholder="••••••••••••"
              class="w-full pl-3 pr-10 py-2 bg-neutral-950 border border-neutral-800 rounded text-sm text-white focus:outline-none focus:border-sky-500" />
            <button
              type="button"
              onclick={() => (showVaultPassword = !showVaultPassword)}
              class="absolute right-2.5 top-1/2 -translate-y-1/2 text-neutral-400 hover:text-white transition-colors p-1"
              aria-label={showVaultPassword ? 'Hide password' : 'Show password'}
              title={showVaultPassword ? 'Hide password' : 'Show password'}
            >
              {#if showVaultPassword}
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
        {#if vaultMessage}
          <p class="text-xs {vaultMessageKind === 'success' ? 'text-emerald-500' : 'text-red-500'}">{vaultMessage}</p>
        {/if}
        <button type="submit" class="px-4 py-2 bg-emerald-600 hover:bg-emerald-500 text-white text-sm font-medium rounded-md transition-colors">
          Update Vault Master Password
        </button>
      </form>
    </div>
  {:else if activeTab === 'backup'}
    <div class="bg-neutral-900 border border-neutral-800 rounded-lg p-6 space-y-8">
      <div>
        <h2 class="text-lg font-semibold text-white">Vault Backup & Restore</h2>
        <p class="text-neutral-400 text-sm mt-1">Export your local vault (hosts, snippets, keys) as an encrypted backup file, or restore from one.</p>
      </div>

      <div class="space-y-4">
        <h3 class="text-base font-semibold text-white">Export Backup</h3>
        <p class="text-xs text-neutral-400">Protects your entire configuration using an AES-256-GCM encryption key derived from your passphrase via Argon2id.</p>
        <form onsubmit={handleExport} class="max-w-md space-y-4 pt-2">
          <div>
            <label for="backup-pass" class="block text-xs font-medium text-neutral-400 uppercase mb-1">Backup Encryption Passphrase (min 8 chars)</label>
            <div class="relative">
              <input
                id="backup-pass"
                type={showBackupPassphrase ? 'text' : 'password'}
                minlength="8"
                required
                bind:value={backupPassphrase}
                class="w-full pl-3 pr-10 py-2 bg-neutral-950 border border-neutral-800 rounded text-sm text-white focus:outline-none focus:border-sky-500" />
              <button
                type="button"
                onclick={() => (showBackupPassphrase = !showBackupPassphrase)}
                class="absolute right-2.5 top-1/2 -translate-y-1/2 text-neutral-400 hover:text-white transition-colors p-1"
                aria-label={showBackupPassphrase ? 'Hide passphrase' : 'Show passphrase'}
                title={showBackupPassphrase ? 'Hide passphrase' : 'Show passphrase'}
              >
                {#if showBackupPassphrase}
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
          {#if backupMsg}
            <p class="text-xs {backupMsgKind === 'success' ? 'text-emerald-500' : 'text-red-500'}">{backupMsg}</p>
          {/if}
          <button type="submit" class="px-4 py-2 bg-sky-600 hover:bg-sky-500 text-white text-sm font-medium rounded-md transition-colors">
            Generate Encrypted Backup...
          </button>
        </form>
      </div>

      <hr class="border-neutral-800" />

      <div class="space-y-4">
        <h3 class="text-base font-semibold text-white">Restore Backup</h3>
        <p class="text-xs text-neutral-400">Restoring merges the backup contents with your current vault. Existing records with the same IDs will be updated.</p>
        <form onsubmit={handleImport} class="max-w-md space-y-4 pt-2">
          <div>
            <label for="restore-pass" class="block text-xs font-medium text-neutral-400 uppercase mb-1">Backup Decryption Passphrase</label>
            <div class="relative">
              <input
                id="restore-pass"
                type={showRestorePassphrase ? 'text' : 'password'}
                required
                bind:value={restorePassphrase}
                class="w-full pl-3 pr-10 py-2 bg-neutral-950 border border-neutral-800 rounded text-sm text-white focus:outline-none focus:border-sky-500" />
              <button
                type="button"
                onclick={() => (showRestorePassphrase = !showRestorePassphrase)}
                class="absolute right-2.5 top-1/2 -translate-y-1/2 text-neutral-400 hover:text-white transition-colors p-1"
                aria-label={showRestorePassphrase ? 'Hide passphrase' : 'Show passphrase'}
                title={showRestorePassphrase ? 'Hide passphrase' : 'Show passphrase'}
              >
                {#if showRestorePassphrase}
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
          {#if restoreMsg}
            <p class="text-xs {restoreMsgKind === 'success' ? 'text-emerald-500' : 'text-red-500'}">{restoreMsg}</p>
          {/if}
          <button type="submit" class="px-4 py-2 bg-amber-600 hover:bg-amber-500 text-white text-sm font-medium rounded-md transition-colors">
            Select Backup File and Restore...
          </button>
        </form>
      </div>
    </div>
  {:else if activeTab === 'shortcuts'}
    <div class="bg-neutral-900 border border-neutral-800 rounded-lg p-6 space-y-6">
      <div>
        <h2 class="text-lg font-semibold text-white">Keyboard Shortcuts</h2>
        <p class="text-neutral-400 text-sm mt-1">Global and tab navigation shortcuts.</p>
      </div>

      <div class="space-y-4">
        <h3 class="text-sm font-semibold text-neutral-300 uppercase tracking-wider">Global</h3>
        <div class="space-y-2">
          <div class="flex justify-between items-center p-3 bg-neutral-950 border border-neutral-800 rounded-md">
            <span class="text-sm text-neutral-200">Command palette</span>
            <kbd class="px-2 py-1 bg-neutral-800 border border-neutral-700 text-neutral-300 text-xs rounded font-mono">Ctrl + K</kbd>
          </div>
        </div>

        <h3 class="text-sm font-semibold text-neutral-300 uppercase tracking-wider pt-2">Sessions</h3>
        <div class="space-y-2">
          <div class="flex justify-between items-center p-3 bg-neutral-950 border border-neutral-800 rounded-md">
            <span class="text-sm text-neutral-200">New session</span>
            <kbd class="px-2 py-1 bg-neutral-800 border border-neutral-700 text-neutral-300 text-xs rounded font-mono">Ctrl + Shift + T</kbd>
          </div>
        </div>

        <h3 class="text-sm font-semibold text-neutral-300 uppercase tracking-wider pt-2">Switch to tab</h3>
        <div class="space-y-2">
          <div class="flex justify-between items-center p-3 bg-neutral-950 border border-neutral-800 rounded-md">
            <span class="text-sm text-neutral-200">Switch to Tab 1</span>
            <kbd class="px-2 py-1 bg-neutral-800 border border-neutral-700 text-neutral-300 text-xs rounded font-mono">Ctrl + 1</kbd>
          </div>
          <div class="flex justify-between items-center p-3 bg-neutral-950 border border-neutral-800 rounded-md">
            <span class="text-sm text-neutral-200">Switch to Tab 2</span>
            <kbd class="px-2 py-1 bg-neutral-800 border border-neutral-700 text-neutral-300 text-xs rounded font-mono">Ctrl + 2</kbd>
          </div>
          <div class="flex justify-between items-center p-3 bg-neutral-950 border border-neutral-800 rounded-md">
            <span class="text-sm text-neutral-200">Switch to Tab 3</span>
            <kbd class="px-2 py-1 bg-neutral-800 border border-neutral-700 text-neutral-300 text-xs rounded font-mono">Ctrl + 3</kbd>
          </div>
        </div>
      </div>
    </div>
  {:else if activeTab === 'feedback'}
    <FeedbackWidget />
  {/if}
</div>