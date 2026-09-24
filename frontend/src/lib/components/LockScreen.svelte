<script lang="ts">
  import { onMount } from 'svelte';
  import { isVaultInitialized, validateVaultPassword, resetVault } from '$lib/api/vault';
  import { showToast, confirmModal } from '$lib/stores/uiNotifications.svelte';
  import Logo from './Logo.svelte';
  import GridFlowBackground from './GridFlowBackground.svelte';
  import ProfileAvatar from './ProfileAvatar.svelte';
  import AvatarPicker from './AvatarPicker.svelte';
  import { getProfile, saveProfile, DEFAULT_AVATAR } from '$lib/stores/profile.svelte';
  import { APP_VERSION } from '$lib/appInfo';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { invoke } from '@tauri-apps/api/core';
  import { t } from '$lib/i18n/index.svelte';
  import LanguageSwitcher from './LanguageSwitcher.svelte';

  export let onUnlocked: () => void;

  let password = '';
  let showPassword = false;
  let errorMsg = '';
  let successMsg = '';
  let isLoading = false;
  let isSetup = false;

  // First-run profile (Free plan: name + preset avatar, no email).
  const profile = getProfile();
  let setupName = '';
  let setupAvatar = DEFAULT_AVATAR;

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

  async function startDragging(e: MouseEvent) {
    if (e.button !== 0) return;
    const target = e.target as HTMLElement | null;
    if (target?.closest('button, input, textarea, a, select, [role="button"], .no-drag')) return;
    try {
      await invoke('window_start_dragging');
    } catch {
      try {
        if (appWindow) {
          await appWindow.startDragging();
        } else if (typeof window !== 'undefined') {
          await getCurrentWindow().startDragging();
        }
      } catch (err) {
        console.warn('Failed to start dragging window:', err);
      }
    }
  }

  // Quote text lives in the dictionaries (`lock.quotes.<key>`) so it follows the chosen language.
  const QUOTES = [
    { key: 'torvalds', author: 'Linus Torvalds' },
    { key: 'jobs', author: 'Steve Jobs' },
    { key: 'hopper', author: 'Grace Hopper' },
    { key: 'dijkstra', author: 'Edsger W. Dijkstra' },
    { key: 'beck', author: 'Kent Beck' }
  ];

  let currentQuote = QUOTES[0];

  onMount(async () => {
  	currentQuote = QUOTES[Math.floor(Math.random() * QUOTES.length)];
  	try {
  		if (appWindow) {
  			const isMax = await appWindow.isMaximized();
  			if (!isMax) {
  				await appWindow.maximize();
  			}
  		}
  	} catch (e) {
  		console.warn('Failed to maximize on mount:', e);
  	}
  	try {
  		isSetup = !(await isVaultInitialized());
  	} catch {
  		isSetup = true; // Fallback to setup if cannot determine
  	}
  });

  async function handleUnlock() {
    if (!password) {
      errorMsg = t('lock.errEmpty');
      return;
    }
    if (password.length < 8) {
      errorMsg = t('lock.errMinLength');
      return;
    }
    isLoading = true;
    errorMsg = '';
    try {
      await validateVaultPassword(password);
      if (isSetup) {
        saveProfile({ name: setupName, avatar: setupAvatar });
        showToast(t('lock.created'), 'success');
      } else {
        showToast(t('lock.unlocked'), 'success');
      }
      onUnlocked();
    } catch (err: any) {
      errorMsg = typeof err === 'string' ? err : (err?.message || t('lock.errInvalid'));
      showToast(errorMsg, 'error');
    } finally {
      isLoading = false;
    }
  }

  async function handleReset() {
    const confirmed = await confirmModal(
      t('lock.resetConfirm'),
      t('lock.resetTitle'),
      true,
      t('lock.resetYes'),
      t('common.cancel')
    );
    if (confirmed) {
      try {
        await resetVault();
        successMsg = t('lock.resetDone');
        showToast(t('lock.resetToast'), 'success');
        isSetup = true;
        password = '';
        errorMsg = '';
      } catch (err: any) {
        showToast(t('lock.resetFailed', { error: String(err) }), 'error');
      }
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      handleUnlock();
    }
  }
</script>

<div
  data-tauri-drag-region
  class="fixed inset-0 z-50 flex bg-neutral-50 dark:bg-[#0a0a0a] text-neutral-900 dark:text-white select-none cursor-default"
  onmousedown={startDragging}
  ondblclick={(e) => {
    const target = e.target as HTMLElement | null;
    if (!target?.closest('button, input, textarea, a, select, [role="button"], .no-drag')) {
      maximizeWindow();
    }
  }}
>
  <!-- Top Drag Region Bar with Custom Window Controls -->
  <div
    data-tauri-drag-region
    class="absolute top-0 left-0 right-0 h-9 z-50 flex items-center justify-between px-3 cursor-default"
    onmousedown={startDragging}
    ondblclick={(e) => {
      const target = e.target as HTMLElement | null;
      if (!target?.closest('button, input, textarea, a, select, [role="button"], .no-drag')) {
        maximizeWindow();
      }
    }}
  >
    <div class="flex items-center gap-2 pointer-events-none opacity-80" data-tauri-drag-region>
      <span class="text-[11px] font-mono text-neutral-500 dark:text-neutral-400 font-semibold tracking-wider" data-tauri-drag-region>CATERM</span>
      <span class="text-[10px] px-1.5 py-0.2 rounded bg-sky-500/15 dark:bg-sky-500/20 text-sky-700 dark:text-sky-400 font-mono" data-tauri-drag-region>v{APP_VERSION}</span>
    </div>
    <!-- Draggable space spanning the rest of the header -->
    <div class="flex-1 h-full" data-tauri-drag-region></div>
    <div class="flex items-center gap-2 no-drag">
      <LanguageSwitcher />
      <button
        onclick={() => minimizeWindow()}
        class="p-1.5 hover:bg-neutral-200 dark:hover:bg-neutral-800 text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white transition-colors cursor-pointer rounded"
        title={t('lock.minimize')}
        aria-label={t('lock.minimize')}
      >
        <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 12H4"></path></svg>
      </button>
      <button
        onclick={() => maximizeWindow()}
        class="p-1.5 hover:bg-neutral-200 dark:hover:bg-neutral-800 text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white transition-colors cursor-pointer rounded"
        title={t('lock.maximize')}
        aria-label={t('lock.maximize')}
      >
        <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><rect x="4" y="4" width="16" height="16" rx="2" stroke-width="2"></rect></svg>
      </button>
      <button
        onclick={() => closeWindow()}
        class="p-1.5 hover:bg-rose-600 hover:text-white text-neutral-500 dark:text-neutral-400 transition-colors cursor-pointer rounded"
        title={t('lock.close')}
        aria-label={t('lock.close')}
      >
        <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path></svg>
      </button>
    </div>
  </div>

  <!-- Left Panel: Brand & Quote -->
  <div
  	data-tauri-drag-region
  	class="hidden lg:flex w-3/4 flex-col justify-between p-12 lg:p-16 xl:p-20 bg-white dark:bg-neutral-950 border-r border-neutral-200 dark:border-neutral-800/60 relative overflow-hidden shrink-0"
  	onmousedown={startDragging}
  >
  	<!-- Sky Blue Grid Flow Animation & CATerm Logo -->
  	<GridFlowBackground targetX={64} targetY={64} count={17} />

  	<div class="relative z-10 flex items-center gap-3">
  		<Logo size={40} mode="brand" />
  		<div>
  			<h1 class="text-xl font-bold tracking-wider text-neutral-900 dark:text-white">CATerm <span class="text-xs px-2 py-0.5 rounded bg-sky-500/10 dark:bg-sky-500/20 text-sky-700 dark:text-sky-400 border border-sky-500/30">v{APP_VERSION}</span></h1>
  			<p class="text-xs text-neutral-500 dark:text-neutral-400">{t('lock.tagline')}</p>
  		</div>
  	</div>

  	<!-- Quote Container -->
  	<div class="relative z-10 my-auto max-w-2xl xl:max-w-3xl">
  		<div class="mb-4 text-sky-500 dark:text-sky-400">
  			<svg class="w-8 h-8 opacity-60" fill="currentColor" viewBox="0 0 24 24"><path d="M14.017 21v-7.391c0-5.704 3.731-9.578 8.983-10.609l.995 2.151c-2.432.917-3.995 3.638-3.995 5.849h4v10h-9.983zm-14.017 0v-7.391c0-5.704 3.748-9.579 9-10.609l.996 2.151c-2.433.917-3.996 3.638-3.996 5.849h3.983v10h-9.983z"/></svg>
  		</div>
  		<blockquote class="text-2xl xl:text-3xl font-light text-neutral-700 dark:text-neutral-200 leading-relaxed italic">
  			"{t(`lock.quotes.${currentQuote.key}`)}"
  		</blockquote>
  		<p class="mt-4 text-sm font-semibold text-sky-600 dark:text-sky-400">— {currentQuote.author}</p>

  		<div class="mt-8 p-5 rounded-xl bg-neutral-50/90 dark:bg-neutral-900/80 border border-neutral-200 dark:border-neutral-800 text-xs text-neutral-600 dark:text-neutral-400 space-y-2">
  			<div class="flex items-center gap-2 text-emerald-600 dark:text-emerald-400 font-medium">
  				<svg class="w-4 h-4 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"/></svg>
  				{t('lock.zkTitle')}
  			</div>
  			<p>{t('lock.zkBody')}</p>
  		</div>
  	</div>

  	<!-- Left Panel Footer -->
  	<div class="relative z-10 flex items-center justify-between text-xs text-neutral-500">
  		<span>{t('lock.zkIdentity')}</span>
  		<span class="font-mono text-neutral-400 dark:text-neutral-600">v{APP_VERSION}</span>
  	</div>
  </div>

  <!-- Right Panel: Master Password Input Form -->
  <div
  	data-tauri-drag-region
  	class="w-full lg:w-1/4 flex flex-col justify-center items-center p-6 sm:p-10 xl:p-12 bg-neutral-50 dark:bg-[#0a0a0a] relative z-10 shrink-0"
  	onmousedown={startDragging}
  >
  	<div class="w-full max-w-sm space-y-8 no-drag">
      <div class="text-center">
        {#if isSetup}
          <div class="w-16 h-16 rounded-full bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 flex items-center justify-center mx-auto mb-4 text-sky-500 dark:text-sky-400">
            <svg class="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"/></svg>
          </div>
          <h2 class="text-2xl font-bold text-neutral-900 dark:text-white">{t('lock.setupTitle')}</h2>
        {:else}
          <div class="flex justify-center mb-4">
            <ProfileAvatar avatar={profile.avatar} name={profile.name} size={64} />
          </div>
          <h2 class="text-2xl font-bold text-neutral-900 dark:text-white">{t('lock.welcomeBack', { name: profile.name })}</h2>
        {/if}
        <p class="text-sm text-neutral-500 dark:text-neutral-400 mt-1">{t('lock.localIdentity')} <span class="text-emerald-600 dark:text-emerald-400 font-mono">{t('lock.encrypted')}</span></p>
      </div>

      {#if successMsg}
        <div class="p-3 text-xs rounded-lg bg-emerald-500/10 border border-emerald-500/30 text-emerald-700 dark:text-emerald-400 text-center">
          {successMsg}
        </div>
      {/if}

      {#if errorMsg}
        <div class="p-3 text-xs rounded-lg bg-rose-500/10 border border-rose-500/30 text-rose-700 dark:text-rose-400 text-center">
          {errorMsg}
        </div>
      {/if}

      <div class="space-y-4">
        {#if isSetup}
          <div>
            <label for="profile-name" class="block text-xs font-semibold uppercase tracking-wider text-neutral-500 dark:text-neutral-400 mb-2">{t('lock.yourName')}</label>
            <input
              id="profile-name"
              type="text"
              bind:value={setupName}
              maxlength="48"
              placeholder="CATerm User"
              class="w-full px-4 py-3 bg-white dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 rounded-lg text-neutral-900 dark:text-white placeholder-neutral-400 dark:placeholder-neutral-600 focus:outline-none focus:border-sky-500 transition-colors"
            />
          </div>
          <div>
            <span class="block text-xs font-semibold uppercase tracking-wider text-neutral-500 dark:text-neutral-400 mb-2">{t('lock.profilePicture')}</span>
            <AvatarPicker bind:value={setupAvatar} size={36} />
          </div>
        {/if}
        <div>
          <label for="master-password" class="block text-xs font-semibold uppercase tracking-wider text-neutral-500 dark:text-neutral-400 mb-2">{isSetup ? t('lock.createMasterPassword') : t('lock.masterPassword')}</label>
          <div class="relative">
            <input
              id="master-password"
              type={showPassword ? 'text' : 'password'}
              bind:value={password}
              onkeydown={handleKeydown}
              placeholder="••••••••••••"
              class="w-full pl-4 pr-11 py-3 bg-white dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 rounded-lg text-neutral-900 dark:text-white placeholder-neutral-400 dark:placeholder-neutral-600 focus:outline-none focus:border-sky-500 transition-colors"
            />
            <button
              type="button"
              onclick={() => (showPassword = !showPassword)}
              class="absolute right-3 top-1/2 -translate-y-1/2 text-neutral-400 hover:text-neutral-900 dark:hover:text-white transition-colors p-1"
              aria-label={showPassword ? t('lock.hidePassword') : t('lock.showPassword')}
              title={showPassword ? t('lock.hidePassword') : t('lock.showPassword')}
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
            {isSetup ? t('lock.settingUp') : t('lock.unlocking')}
          {:else}
            {isSetup ? t('lock.createVault') : t('lock.unlockVault')}
          {/if}
        </button>
      </div>

      <div class="flex items-center justify-between text-xs text-neutral-500 pt-4 border-t border-neutral-200 dark:border-neutral-900">
        {#if !isSetup}
          <button onclick={handleReset} class="hover:text-rose-600 dark:hover:text-rose-400 transition-colors cursor-pointer">
            {t('lock.resetVault')}
          </button>
        {:else}
          <span></span>
        {/if}
        <span>{t('lock.vaultEncrypted')}</span>
      </div>
    </div>
  </div>
</div>
