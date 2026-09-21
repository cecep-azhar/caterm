<script lang="ts">
  import { onMount } from 'svelte';
  import { isVaultInitialized, validateVaultPassword, resetVault } from '$lib/api/vault';
  import { showToast, confirmModal } from '$lib/stores/uiNotifications.svelte';
  import Logo from './Logo.svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { invoke } from '@tauri-apps/api/core';

  export let onUnlocked: () => void;

  let password = '';
  let showPassword = false;
  let errorMsg = '';
  let successMsg = '';
  let isLoading = false;
  let isSetup = false;

  const isTauri = typeof window !== 'undefined' && Boolean((window as any).__TAURI_INTERNALS__ || (window as any).__TAURI__);
  const appWindow = isTauri ? getCurrentWindow() : null;

  async function minimizeWindow() {
    try {
      await invoke('window_minimize');
    } catch {
      try {
        if (appWindow) {
          await appWindow.minimize();
        } else if (typeof window !== 'undefined') {
          await getCurrentWindow().minimize();
        }
      } catch (err) {
        console.warn('Failed to minimize window:', err);
      }
    }
  }

  async function maximizeWindow() {
    try {
      await invoke('window_maximize');
    } catch {
      try {
        if (appWindow) {
          await appWindow.toggleMaximize();
        } else if (typeof window !== 'undefined') {
          await getCurrentWindow().toggleMaximize();
        }
      } catch (err) {
        console.warn('Failed to toggle maximize window:', err);
      }
    }
  }

  async function closeWindow() {
    try {
      await invoke('window_close');
    } catch {
      try {
        if (appWindow) {
          await appWindow.close();
        } else if (typeof window !== 'undefined') {
          await getCurrentWindow().close();
        }
      } catch (err) {
        console.warn('Failed to close window:', err);
      }
    }
  }

  const QUOTES = [
    { text: "Talk is cheap. Show me the code.", author: "Linus Torvalds" },
    { text: "The only way to do great work is to love what you do.", author: "Steve Jobs" },
    { text: "The most dangerous phrase in the language is, 'We've always done it this way.'", author: "Grace Hopper" },
    { text: "Simplicity is prerequisite for reliability.", author: "Edsger W. Dijkstra" },
    { text: "Make it work, make it right, make it fast.", author: "Kent Beck" }
  ];

  let currentQuote = QUOTES[0];

  onMount(async () => {
    currentQuote = QUOTES[Math.floor(Math.random() * QUOTES.length)];
    try {
      isSetup = !(await isVaultInitialized());
    } catch {
      isSetup = true; // Fallback to setup if cannot determine
    }
  });

  async function handleUnlock() {
    if (!password) {
      errorMsg = 'Master password cannot be empty.';
      return;
    }
    if (password.length < 8) {
      errorMsg = 'Master password must be at least 8 characters.';
      return;
    }
    isLoading = true;
    errorMsg = '';
    try {
      await validateVaultPassword(password);
      if (isSetup) {
        showToast('Vault successfully created and encrypted.', 'success');
      } else {
        showToast('Vault successfully unlocked.', 'success');
      }
      onUnlocked();
    } catch (err: any) {
      errorMsg = typeof err === 'string' ? err : (err?.message || 'Invalid password (minimum 8 characters)');
      showToast(errorMsg, 'error');
    } finally {
      isLoading = false;
    }
  }

  async function handleReset() {
    const confirmed = await confirmModal(
      'Resetting the Vault will permanently delete master password, local key, and all stored data. Are you sure?',
      'Reset Local Vault',
      true,
      'Yes, Reset Vault',
      'Cancel'
    );
    if (confirmed) {
      try {
        await resetVault();
        successMsg = 'Vault reset successfully! All database and local keys have been cleared.';
        showToast('Vault reset successfully! All database records cleared.', 'success');
        isSetup = true;
        password = '';
        errorMsg = '';
      } catch (err: any) {
        showToast('Failed to reset vault: ' + String(err), 'error');
      }
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      handleUnlock();
    }
  }
</script>

<div class="fixed inset-0 z-50 flex bg-[#0a0a0a] text-white select-none">
  <!-- Top Drag Region Bar with Custom Window Controls -->
  <div data-tauri-drag-region class="absolute top-0 left-0 right-0 h-9 z-50 flex items-center justify-between px-3">
    <div class="flex items-center gap-2 pointer-events-none opacity-80">
      <span class="text-[11px] font-mono text-neutral-400 font-semibold tracking-wider">CATERM</span>
      <span class="text-[10px] px-1.5 py-0.2 rounded bg-sky-500/20 text-sky-400 font-mono">v2.1.6</span>
    </div>
    <div class="flex items-center no-drag">
      <button
        onclick={() => minimizeWindow()}
        class="p-1.5 hover:bg-neutral-800 text-neutral-400 hover:text-white transition-colors cursor-pointer rounded"
        title="Minimize"
        aria-label="Minimize"
      >
        <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 12H4"></path></svg>
      </button>
      <button
        onclick={() => maximizeWindow()}
        class="p-1.5 hover:bg-neutral-800 text-neutral-400 hover:text-white transition-colors cursor-pointer rounded"
        title="Maximize"
        aria-label="Maximize"
      >
        <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><rect x="4" y="4" width="16" height="16" rx="2" stroke-width="2"></rect></svg>
      </button>
      <button
        onclick={() => closeWindow()}
        class="p-1.5 hover:bg-rose-600 hover:text-white text-neutral-400 transition-colors cursor-pointer rounded"
        title="Close"
        aria-label="Close"
      >
        <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path></svg>
      </button>
    </div>
  </div>

  <!-- Left Panel: Brand & Quote -->
  <div class="hidden lg:flex flex-1 flex-col justify-between p-12 bg-neutral-950 border-r border-neutral-800/60 relative overflow-hidden">
    <!-- Ambient Grid Effect -->
    <div class="absolute inset-0 bg-[linear-gradient(to_right,#1f293715_1px,transparent_1px),linear-gradient(to_bottom,#1f293715_1px,transparent_1px)] bg-[size:4rem_4rem]"></div>
    
    <div class="relative z-10 flex items-center gap-3">
      <Logo size={40} mode="brand" />
      <div>
        <h1 class="text-xl font-bold tracking-wider text-white">CATerm <span class="text-xs px-2 py-0.5 rounded bg-sky-500/20 text-sky-400 border border-sky-500/30">v2.1.6</span></h1>
        <p class="text-xs text-neutral-400">Enterprise SSH Manager & Prompt Studio</p>
      </div>
    </div>

    <!-- Quote Container -->
    <div class="relative z-10 my-auto max-w-lg">
      <div class="mb-4 text-sky-400">
        <svg class="w-8 h-8 opacity-60" fill="currentColor" viewBox="0 0 24 24"><path d="M14.017 21v-7.391c0-5.704 3.731-9.57 8.983-10.609l.995 2.151c-2.432.917-3.995 3.638-3.995 5.849h4v10h-9.983zm-14.017 0v-7.391c0-5.704 3.748-9.57 9-10.609l.996 2.151c-2.433.917-3.996 3.638-3.996 5.849h3.983v10h-9.983z"/></svg>
      </div>
      <blockquote class="text-2xl font-light text-neutral-200 leading-relaxed italic">
        "{currentQuote.text}"
      </blockquote>
      <p class="mt-4 text-sm font-semibold text-sky-400">— {currentQuote.author}</p>

      <div class="mt-8 p-4 rounded-xl bg-neutral-900/80 border border-neutral-800 text-xs text-neutral-400 space-y-2">
        <div class="flex items-center gap-2 text-emerald-400 font-medium">
          <svg class="w-4 h-4 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"/></svg>
          Zero-Knowledge Local Encryption (Argon2id + AES-GCM)
        </div>
        <p>Your vault credentials and SSH private keys are encrypted locally. We never store or transmit your master password to any cloud server.</p>
      </div>
    </div>

    <!-- Dedication Footer -->
    <div class="relative z-10 text-xs text-neutral-500">
      Dedication: <span class="text-neutral-300 font-medium italic">To the Five Lights of My Life: Fatih, Harun, Ibrahim, Khadijah, Maryam, and My Beloved Wife Rini..</span>
    </div>
  </div>

  <!-- Right Panel: Master Password Input Form -->
  <div class="flex-1 flex flex-col justify-center items-center p-8 sm:p-16 bg-[#0a0a0a]">
    <div class="w-full max-w-md space-y-8">
      <div class="text-center">
        <div class="w-16 h-16 rounded-full bg-neutral-900 border border-neutral-800 flex items-center justify-center mx-auto mb-4 text-sky-400">
          <svg class="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"/></svg>
        </div>
        <h2 class="text-2xl font-bold text-white">{isSetup ? 'Setup CATerm Vault' : 'Unlock CATerm Vault'}</h2>
        <p class="text-sm text-neutral-400 mt-1">Local Identity <span class="text-emerald-400 font-mono">(Encrypted)</span></p>
      </div>

      {#if successMsg}
        <div class="p-3 text-xs rounded-lg bg-emerald-500/10 border border-emerald-500/30 text-emerald-400 text-center">
          {successMsg}
        </div>
      {/if}

      {#if errorMsg}
        <div class="p-3 text-xs rounded-lg bg-rose-500/10 border border-rose-500/30 text-rose-400 text-center">
          {errorMsg}
        </div>
      {/if}

      <div class="space-y-4">
        <div>
          <label for="master-password" class="block text-xs font-semibold uppercase tracking-wider text-neutral-400 mb-2">{isSetup ? 'Create Master Password' : 'Master Password'}</label>
          <div class="relative">
            <input
              id="master-password"
              type={showPassword ? 'text' : 'password'}
              bind:value={password}
              onkeydown={handleKeydown}
              placeholder="••••••••••••"
              class="w-full pl-4 pr-11 py-3 bg-neutral-900 border border-neutral-800 rounded-lg text-white placeholder-neutral-600 focus:outline-none focus:border-sky-500 transition-colors"
            />
            <button
              type="button"
              onclick={() => (showPassword = !showPassword)}
              class="absolute right-3 top-1/2 -translate-y-1/2 text-neutral-400 hover:text-white transition-colors p-1"
              aria-label={showPassword ? 'Hide password' : 'Show password'}
              title={showPassword ? 'Hide password' : 'Show password'}
            >
              {#if showPassword}
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21" />
                </svg>
              {:else}
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                </svg>
              {/if}
            </button>
          </div>
        </div>

        <button
          onclick={handleUnlock}
          disabled={isLoading}
          class="w-full py-3 bg-sky-600 hover:bg-sky-500 disabled:opacity-50 text-white font-medium rounded-lg shadow-lg shadow-sky-600/20 transition-all flex items-center justify-center gap-2 text-sm"
        >
          {#if isLoading}
            <svg class="w-4 h-4 animate-spin text-white" fill="none" viewBox="0 0 24 24"><circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle><path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path></svg>
            {isSetup ? 'Setting up...' : 'Unlocking...'}
          {:else}
            {isSetup ? 'Create Vault' : 'Unlock Vault'}
          {/if}
        </button>
      </div>

      <div class="flex items-center justify-between text-xs text-neutral-500 pt-4 border-t border-neutral-900">
        {#if !isSetup}
          <button onclick={handleReset} class="hover:text-rose-400 transition-colors cursor-pointer">
            Reset Vault
          </button>
        {:else}
          <span></span>
        {/if}
        <span>Local Vault: Encrypted</span>
      </div>
    </div>
  </div>
</div>
