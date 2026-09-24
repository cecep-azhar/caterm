<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import { changeMasterPassword, MIN_VAULT_PASSWORD_LEN } from '$lib/api/vault';
  import { exportEncryptedBackup, importEncryptedBackup } from '$lib/api/backup';
  import AiSettingsForm from '$lib/components/AiSettingsForm.svelte';
  import ProfileAvatar from '$lib/components/ProfileAvatar.svelte';
  import AvatarPicker from '$lib/components/AvatarPicker.svelte';
  import { showToast, confirmModal } from '$lib/stores/uiNotifications.svelte';
  import { relaunch } from '@tauri-apps/plugin-process';
  import { getProfile, saveProfile } from '$lib/stores/profile.svelte';
  import { getUpdater, checkForUpdates, installUpdate } from '$lib/stores/updater.svelte';
  import { APP_VERSION, releaseNotesUrl } from '$lib/appInfo';
  import { errorText } from '$lib/errors';

  const TABS = ['profile', 'updates', 'ai', 'subscription', 'sync', 'security', 'backup', 'shortcuts'];
  // `?tab=` lets the profile menu deep-link straight to a tab.
  const requestedTab = page.url.searchParams.get('tab') ?? '';
  let activeTab = $state(TABS.includes(requestedTab) ? requestedTab : 'profile');

  const updater = getUpdater();
  const downloadPercent = $derived(
    updater.progress.total ? Math.round((updater.progress.downloaded / updater.progress.total) * 100) : null
  );

  // Profile (Free plan: display name + preset avatar)
  const profile = getProfile();
  let profileName = $state(profile.name);
  let profileAvatar = $state(profile.avatar);
  const profileDirty = $derived(profileName.trim() !== profile.name || profileAvatar !== profile.avatar);

  function handleSaveProfile(e: Event) {
    e.preventDefault();
    saveProfile({ name: profileName, avatar: profileAvatar });
    profileName = profile.name;
    showToast('Profile updated.', 'success');
  }

  // Master password change (re-keys the encrypted database)
  let currentPassword = $state('');
  let newPassword = $state('');
  let confirmPassword = $state('');
  let showPasswords = $state(false);
  let isChangingPassword = $state(false);

  // Audit Logs Setting
  let maxAuditRecords = $state(1000);

  onMount(() => {
    try {
      const stored = localStorage.getItem('caterm_max_audit_records');
      if (stored) maxAuditRecords = parseInt(stored, 10) || 1000;
    } catch {}
  });

  function handleSaveAuditSettings(e: Event) {
    e.preventDefault();
    try {
      localStorage.setItem('caterm_max_audit_records', maxAuditRecords.toString());
      showToast(`Max audit records set to ${maxAuditRecords}`, 'success');
    } catch {
      showToast('Failed to save audit settings', 'error');
    }
  }

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

  async function handleChangeMasterPassword(e: Event) {
    e.preventDefault();
    if (newPassword.length < MIN_VAULT_PASSWORD_LEN) {
      showToast(`New master password must be at least ${MIN_VAULT_PASSWORD_LEN} characters.`, 'error');
      return;
    }
    if (newPassword !== confirmPassword) {
      showToast('New password and confirmation do not match.', 'error');
      return;
    }
    isChangingPassword = true;
    try {
      await changeMasterPassword(currentPassword, newPassword);
      currentPassword = '';
      newPassword = '';
      confirmPassword = '';
      const restartNow = await confirmModal(
        'Master password changed and the vault re-encrypted. Restart CATerm now and unlock with the new password?',
        'Master Password Changed',
        false,
        'Restart now',
        'Later'
      );
      if (restartNow) {
        await relaunch();
        return;
      }
      showToast('Master password changed. Use the new password next time you unlock.', 'success');
    } catch (err) {
      showToast(errorText(err), 'error');
    } finally {
      isChangingPassword = false;
    }
  }
</script>

<div class="max-w-4xl mx-auto space-y-6">
  <div class="pb-4 border-b border-neutral-200 dark:border-neutral-800/80 mb-6">
    <h1 class="text-2xl font-bold text-neutral-900 dark:text-white tracking-tight">Settings</h1>
    <p class="text-neutral-500 dark:text-neutral-400 text-sm mt-1">Configure global preferences, system updates, and zero-knowledge local security.</p>
  </div>

  <!-- Settings Tabs -->
  <div class="border-b border-neutral-200 dark:border-neutral-800 flex gap-4 overflow-x-auto">
    <button
      onclick={() => activeTab = 'profile'}
      class="pb-3 whitespace-nowrap text-sm font-medium transition-colors border-b-2 {activeTab === 'profile' ? 'border-sky-500 text-neutral-900 dark:text-white font-semibold' : 'border-transparent text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200'}">
      Profile
    </button>
    <button 
      onclick={() => activeTab = 'updates'} 
      class="pb-3 whitespace-nowrap text-sm font-medium transition-colors border-b-2 {activeTab === 'updates' ? 'border-sky-500 text-neutral-900 dark:text-white font-semibold' : 'border-transparent text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200'}">
      Updates
    </button>
    <button
      onclick={() => activeTab = 'ai'}
      class="pb-3 whitespace-nowrap text-sm font-medium transition-colors border-b-2 {activeTab === 'ai' ? 'border-sky-500 text-neutral-900 dark:text-white font-semibold' : 'border-transparent text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200'}">
      AI Assistant
    </button>
    <button
      onclick={() => activeTab = 'subscription'}
      class="pb-3 whitespace-nowrap text-sm font-medium transition-colors border-b-2 {activeTab === 'subscription' ? 'border-sky-500 text-neutral-900 dark:text-white font-semibold' : 'border-transparent text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200'}">
      Subscription
    </button>
    <button
      onclick={() => activeTab = 'sync'}
      class="pb-3 whitespace-nowrap text-sm font-medium transition-colors border-b-2 {activeTab === 'sync' ? 'border-sky-500 text-neutral-900 dark:text-white font-semibold' : 'border-transparent text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200'}">
      Cloud Sync E2EE
    </button>
    <button 
      onclick={() => activeTab = 'security'} 
      class="pb-3 whitespace-nowrap text-sm font-medium transition-colors border-b-2 {activeTab === 'security' ? 'border-sky-500 text-neutral-900 dark:text-white font-semibold' : 'border-transparent text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200'}">
      Vault Security
    </button>
    <button 
      onclick={() => activeTab = 'backup'} 
      class="pb-3 whitespace-nowrap text-sm font-medium transition-colors border-b-2 {activeTab === 'backup' ? 'border-sky-500 text-neutral-900 dark:text-white font-semibold' : 'border-transparent text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200'}">
      Backup & Restore
    </button>
    <button 
      onclick={() => activeTab = 'shortcuts'} 
      class="pb-3 whitespace-nowrap text-sm font-medium transition-colors border-b-2 {activeTab === 'shortcuts' ? 'border-sky-500 text-neutral-900 dark:text-white font-semibold' : 'border-transparent text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200'}">
      Shortcuts
    </button>
  </div>

  {#if activeTab === 'profile'}
    <div class="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-lg p-6 space-y-6 shadow-sm text-neutral-900 dark:text-white">
      <div class="flex items-center gap-4">
        <ProfileAvatar avatar={profileAvatar} name={profileName} size={56} />
        <div class="min-w-0">
          <div class="flex items-center gap-2">
            <h2 class="text-lg font-semibold text-neutral-900 dark:text-white truncate">{profileName.trim() || profile.name}</h2>
            <span class="text-[10px] font-semibold tracking-wider px-1.5 py-0.5 rounded border border-neutral-300 dark:border-neutral-700 text-neutral-600 dark:text-neutral-400">FREE</span>
          </div>
          <p class="text-neutral-500 dark:text-neutral-400 text-sm">Local profile — stored on this device only, no account or email needed.</p>
        </div>
      </div>

      <form onsubmit={handleSaveProfile} class="space-y-5 max-w-md">
        <div>
          <label for="profile-name" class="block text-xs font-medium text-neutral-600 dark:text-neutral-400 uppercase mb-1">Display name</label>
          <input
            id="profile-name"
            type="text"
            maxlength="48"
            bind:value={profileName}
            placeholder="CATerm User"
            class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded text-sm text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500" />
        </div>
        <div>
          <span class="block text-xs font-medium text-neutral-600 dark:text-neutral-400 uppercase mb-2">Profile picture</span>
          <AvatarPicker bind:value={profileAvatar} size={40} />
        </div>
        <button
          type="submit"
          disabled={!profileDirty}
          class="px-4 py-2 bg-sky-600 hover:bg-sky-500 disabled:opacity-40 disabled:hover:bg-sky-600 text-white text-sm font-medium rounded-md transition-colors shadow-sm">
          Save profile
        </button>
      </form>

      <div class="pt-5 border-t border-neutral-200 dark:border-neutral-800 flex items-center justify-between gap-4">
        <div>
          <p class="text-sm font-medium text-neutral-900 dark:text-white">Master password</p>
          <p class="text-xs text-neutral-500 dark:text-neutral-400">Unlocks and encrypts your local vault.</p>
        </div>
        <button
          onclick={() => (activeTab = 'security')}
          class="px-3 py-1.5 text-xs font-medium rounded-md border border-neutral-300 dark:border-neutral-700 text-neutral-700 dark:text-neutral-200 hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors">
          Change master password
        </button>
      </div>
    </div>
  {:else if activeTab === 'updates'}
    <div class="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-lg p-6 space-y-4 shadow-sm text-neutral-900 dark:text-white">
      <div class="flex flex-wrap justify-between items-center gap-4">
        <div>
          <h2 class="text-lg font-semibold text-neutral-900 dark:text-white">Application Updates</h2>
          <p class="text-neutral-500 dark:text-neutral-400 text-sm">Current installed version: <span class="font-mono text-sky-600 dark:text-sky-400">v{APP_VERSION}</span></p>
        </div>
        {#if updater.status === 'available'}
          <button
            onclick={installUpdate}
            class="px-4 py-2 bg-emerald-600 hover:bg-emerald-500 text-white text-sm font-medium rounded-md transition-colors shadow-sm">
            Download and Install Update
          </button>
        {:else}
          <button
            onclick={() => checkForUpdates()}
            disabled={updater.status === 'checking' || updater.status === 'downloading'}
            class="px-4 py-2 bg-sky-600 hover:bg-sky-500 disabled:opacity-60 text-white text-sm font-medium rounded-md transition-colors flex items-center gap-2 shadow-sm">
            {#if updater.status === 'checking'}
              <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24"><circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle><path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path></svg>
              Checking for updates...
            {:else}
              Check for Updates
            {/if}
          </button>
        {/if}
      </div>

      {#if updater.status === 'up-to-date'}
        <div class="flex items-center gap-2 p-3 rounded-md bg-emerald-500/10 border border-emerald-500/30 text-emerald-400 text-sm">
          <svg class="w-4 h-4 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" /></svg>
          You are on the latest version (v{APP_VERSION}).
        </div>
      {:else if updater.status === 'available'}
        <div class="p-3 rounded-md bg-sky-500/10 border border-sky-500/30 text-sky-300 text-sm">
          Version <span class="font-mono">v{updater.version}</span> is available.
          <a href={releaseNotesUrl(updater.version)} class="underline underline-offset-2 hover:text-white">Read the changelog</a>
        </div>
      {:else if updater.status === 'downloading'}
        <div class="p-3 rounded-md bg-sky-500/10 border border-sky-500/30 text-sky-300 text-sm">
          Downloading v{updater.version}{downloadPercent === null ? '...' : ` (${downloadPercent}%)`} — CATerm restarts when the install finishes.
        </div>
      {:else if updater.status === 'error'}
        <div class="p-3 rounded-md bg-rose-500/10 border border-rose-500/30 text-rose-300 text-sm">
          {updater.error}
        </div>
      {/if}
    </div>
  {:else if activeTab === 'ai'}
    <div class="bg-neutral-900 border border-neutral-800 rounded-lg p-6 space-y-4">
      <div>
        <h2 class="text-lg font-semibold text-white">AI Ops Assistant</h2>
        <p class="text-neutral-400 text-sm">
          LLM endpoint, credentials, and model used by AI Chat and Prompt Studio.
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

      <form onsubmit={handleChangeMasterPassword} class="max-w-md space-y-4 pt-2">
        <p class="text-xs text-neutral-400">
          Changing the master password re-encrypts the whole local database with a key derived from the new password.
          Keep the app open until it finishes.
        </p>
        <div>
          <label for="current-pass" class="block text-xs font-medium text-neutral-400 uppercase mb-1">Current master password</label>
          <input
            id="current-pass"
            type={showPasswords ? 'text' : 'password'}
            required
            autocomplete="current-password"
            bind:value={currentPassword}
            class="w-full px-3 py-2 bg-neutral-950 border border-neutral-800 rounded text-sm text-white focus:outline-none focus:border-sky-500" />
        </div>
        <div>
          <label for="new-pass" class="block text-xs font-medium text-neutral-400 uppercase mb-1">New master password (min. {MIN_VAULT_PASSWORD_LEN} chars)</label>
          <input
            id="new-pass"
            type={showPasswords ? 'text' : 'password'}
            required
            minlength={MIN_VAULT_PASSWORD_LEN}
            autocomplete="new-password"
            bind:value={newPassword}
            class="w-full px-3 py-2 bg-neutral-950 border border-neutral-800 rounded text-sm text-white focus:outline-none focus:border-sky-500" />
        </div>
        <div>
          <label for="confirm-pass" class="block text-xs font-medium text-neutral-400 uppercase mb-1">Confirm new master password</label>
          <input
            id="confirm-pass"
            type={showPasswords ? 'text' : 'password'}
            required
            minlength={MIN_VAULT_PASSWORD_LEN}
            autocomplete="new-password"
            bind:value={confirmPassword}
            class="w-full px-3 py-2 bg-neutral-950 border border-neutral-800 rounded text-sm text-white focus:outline-none focus:border-sky-500" />
        </div>
        <label class="flex items-center gap-2 text-xs text-neutral-400">
          <input type="checkbox" bind:checked={showPasswords} class="rounded" />
          Show passwords
        </label>
        <button
          type="submit"
          disabled={isChangingPassword}
          class="px-4 py-2 bg-emerald-600 hover:bg-emerald-500 disabled:opacity-60 text-white text-sm font-medium rounded-md transition-colors">
          {isChangingPassword ? 'Re-encrypting vault...' : 'Change Master Password'}
        </button>
      </form>

      <!-- Audit Trail Settings -->
      <div class="border-t border-neutral-800 pt-6 space-y-3">
        <div>
          <h3 class="text-base font-semibold text-white">Audit Trail & Command Logs</h3>
          <p class="text-neutral-400 text-xs mt-1">Limit the number of records loaded from the local database. Older records beyond the limit are stored but not shown until the limit is raised.</p>
        </div>
        <form onsubmit={handleSaveAuditSettings} class="flex items-end gap-4 max-w-sm">
          <div class="flex-1">
            <label for="max-audit" class="block text-xs font-medium text-neutral-400 uppercase mb-1">Max Records (default: 1000)</label>
            <input
              id="max-audit"
              type="number"
              min="100"
              max="50000"
              step="100"
              bind:value={maxAuditRecords}
              class="w-full px-3 py-2 bg-neutral-950 border border-neutral-800 rounded text-sm text-white focus:outline-none focus:border-sky-500"
            />
          </div>
          <button
            type="submit"
            class="px-4 py-2 bg-sky-600 hover:bg-sky-500 text-white text-sm font-medium rounded-md transition-colors whitespace-nowrap"
          >
            Save
          </button>
        </form>
      </div>
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
  {/if}
</div>