<script lang="ts">
  import { validateVaultPassword, MIN_VAULT_PASSWORD_LEN } from '$lib/api/vault';

  let activeTab = $state('updates'); // 'updates' | 'subscription' | 'sync' | 'security'
  let vaultPassword = $state('');
  let vaultMessage = $state('');
  let vaultMessageKind = $state<'error' | 'success'>('error');

  async function updateVaultPassword(e: Event) {
    e.preventDefault();
    try {
      await validateVaultPassword(vaultPassword);
      vaultMessageKind = 'success';
      vaultMessage = 'Master password valid dan tersimpan.';
      vaultPassword = '';
    } catch (err) {
      vaultMessageKind = 'error';
      vaultMessage =
        (err as { message?: string })?.message ??
        `Password vault minimal ${MIN_VAULT_PASSWORD_LEN} karakter.`;
    }
  }
</script>

<div class="max-w-4xl mx-auto space-y-6">
  <div>
    <h1 class="text-2xl font-bold text-white tracking-tight">Settings</h1>
    <p class="text-neutral-400 text-sm mt-1">Configure global preferences, system updates, and zero-knowledge local security.</p>
  </div>

  <!-- Settings Tabs -->
  <div class="border-b border-neutral-800 flex gap-4 overflow-x-auto">
    <button 
      onclick={() => activeTab = 'updates'} 
      class="pb-3 whitespace-nowrap text-sm font-medium transition-colors border-b-2 {activeTab === 'updates' ? 'border-sky-500 text-white' : 'border-transparent text-neutral-400 hover:text-neutral-200'}">
      Updates
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
      onclick={() => activeTab = 'shortcuts'} 
      class="pb-3 whitespace-nowrap text-sm font-medium transition-colors border-b-2 {activeTab === 'shortcuts' ? 'border-sky-500 text-white' : 'border-transparent text-neutral-400 hover:text-neutral-200'}">
      Shortcuts
    </button>
  </div>

  {#if activeTab === 'updates'}
    <div class="bg-neutral-900 border border-neutral-800 rounded-lg p-6 space-y-4">
      <div class="flex justify-between items-center">
        <div>
          <h2 class="text-lg font-semibold text-white">Application Updates</h2>
          <p class="text-neutral-400 text-sm">Current installed version: <span class="font-mono text-sky-400">v2.0.2</span></p>
        </div>
        <button class="px-4 py-2 bg-sky-600 hover:bg-sky-500 text-white text-sm font-medium rounded-md transition-colors">
          Check for Updates
        </button>
      </div>
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
          <p class="text-neutral-400 text-xs mt-2">Limited to <span class="text-neutral-200 font-medium">3 hosts</span> and <span class="text-neutral-200 font-medium">10 snippets</span>.</p>
        </div>
      </div>

      <div class="border border-sky-500/30 rounded-lg p-5 bg-neutral-950 flex flex-col gap-4 opacity-90">
        <div class="flex justify-between items-start">
          <div>
            <div class="flex items-center gap-2">
              <span class="font-semibold text-white">Pro Plan</span>
              <span class="text-xs bg-amber-500/10 text-amber-400 border border-amber-500/20 px-2 py-0.5 rounded">Next Feature / Disabled</span>
            </div>
            <p class="text-neutral-400 text-xs mt-2">Unlimited hosts &amp; snippets, priority support, and team collaboration.</p>
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
          <input
            id="vault-pass"
            type="password"
            minlength={MIN_VAULT_PASSWORD_LEN}
            required
            bind:value={vaultPassword}
            placeholder="••••••••••••"
            class="w-full px-3 py-2 bg-neutral-950 border border-neutral-800 rounded text-sm text-white focus:outline-none focus:border-sky-500" />
        </div>
        {#if vaultMessage}
          <p class="text-xs {vaultMessageKind === 'success' ? 'text-emerald-500' : 'text-red-500'}">{vaultMessage}</p>
        {/if}
        <button type="submit" class="px-4 py-2 bg-emerald-600 hover:bg-emerald-500 text-white text-sm font-medium rounded-md transition-colors">
          Update Vault Master Password
        </button>
      </form>
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