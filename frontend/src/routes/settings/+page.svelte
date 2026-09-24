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
  import { APP_VERSION, releaseNotesUrl, PRICING_URL } from '$lib/appInfo';
  import { PRO_PRICING, formatUsd, type BillingInterval } from '$lib/pro/pricing';
  import { openExternalUrl } from '$lib/utils/url';
  import { errorText } from '$lib/errors';
  import { getPerformancePrefs, setPerformancePrefs } from '$lib/api/performance';
  import { t } from '$lib/i18n/index.svelte';
  import PageHeader from '$lib/components/PageHeader.svelte';

  // Shared surface classes so every tab reads the same in light and dark mode.
  const CARD = 'bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-lg p-6 shadow-sm dark:shadow-none text-neutral-900 dark:text-white';
  const SUBCARD = 'border border-neutral-200 dark:border-neutral-800 rounded-lg p-5 bg-neutral-50 dark:bg-neutral-950';
  const MUTED = 'text-neutral-500 dark:text-neutral-400';
  const LABEL_BASE = 'block text-xs font-medium text-neutral-600 dark:text-neutral-400 uppercase';
  const LABEL = `${LABEL_BASE} mb-1`;
  const INPUT = 'w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded text-sm text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500';
  const EYE_BUTTON = 'absolute right-2.5 top-1/2 -translate-y-1/2 text-neutral-400 hover:text-neutral-900 dark:hover:text-white transition-colors p-1';

  // Subscription tab
  let billing = $state<BillingInterval>('yearly');
  const proFeatures = $derived([
    t('settings.subscription.featureSync', { devices: PRO_PRICING.syncDevices }),
    t('settings.subscription.featureAi', { requests: PRO_PRICING.aiRequestsPerMonth }),
    t('settings.subscription.featureTeam', { extra: PRO_PRICING.maxMembers - 1, members: PRO_PRICING.maxMembers }),
    t('settings.subscription.featureLogs'),
    t('settings.subscription.featureCommunity')
  ]);

  const SHORTCUT_GROUPS: { title: string; items: { label: string; keys: string; vars?: Record<string, number> }[] }[] = [
    { title: 'global', items: [{ label: 'commandPalette', keys: 'Ctrl + K' }] },
    { title: 'sessions', items: [{ label: 'newSession', keys: 'Ctrl + Shift + T' }] },
    {
      title: 'switchTab',
      items: [1, 2, 3].map((n) => ({ label: 'switchToTab', keys: `Ctrl + ${n}`, vars: { n } }))
    }
  ];

  const TABS = ['profile', 'updates', 'ai', 'subscription', 'security', 'backup', 'performance', 'shortcuts'] as const;
  type SettingsTab = (typeof TABS)[number];
  // `?tab=` lets the profile menu deep-link straight to a tab.
  const requestedTab = page.url.searchParams.get('tab') ?? '';
  let activeTab = $state<SettingsTab>((TABS as readonly string[]).includes(requestedTab) ? (requestedTab as SettingsTab) : 'profile');

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
    showToast(t('settings.profile.saved'), 'success');
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
    getPerformancePrefs()
      .then((prefs) => (gpuAcceleration = prefs.gpuAcceleration))
      .catch(() => {}); // browser preview: no backend, keep the default
  });

  // Performance: GPU on/off is read when the window is created, so it needs a restart.
  let gpuAcceleration = $state(true);
  let isSavingPerformance = $state(false);

  async function toggleGpuAcceleration() {
    const next = !gpuAcceleration;
    isSavingPerformance = true;
    try {
      await setPerformancePrefs({ gpuAcceleration: next });
      gpuAcceleration = next;
    } catch (err) {
      showToast(t('settings.performance.saveFailed', { error: errorText(err) }), 'error');
      return;
    } finally {
      isSavingPerformance = false;
    }
    const restart = await confirmModal(
      t('settings.performance.restartBody'),
      t('settings.performance.restartTitle'),
      false,
      t('settings.performance.restartNow'),
      t('settings.performance.later')
    );
    if (restart) await relaunch();
  }

  function handleSaveAuditSettings(e: Event) {
    e.preventDefault();
    try {
      localStorage.setItem('caterm_max_audit_records', maxAuditRecords.toString());
      showToast(t('settings.audit.saved', { count: maxAuditRecords }), 'success');
    } catch {
      showToast(t('settings.audit.saveFailed'), 'error');
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
      showToast(t('settings.backup.errPassphrase'), 'error');
      return;
    }

    try {
      showToast(t('settings.backup.creating'), 'info');
      const b64 = await exportEncryptedBackup(backupPassphrase);
      
      const blob = new Blob([b64], { type: 'text/plain' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `caterm-backup-${new Date().toISOString().split('T')[0]}.catb`;
      a.click();
      URL.revokeObjectURL(url);

      showToast(t('settings.backup.savedToast'), 'success');
      backupPassphrase = '';
    } catch (err: any) {
      showToast(err?.message || t('settings.backup.exportFailed'), 'error');
    }
  }

  async function handleImport(e: Event) {
    e.preventDefault();
    if (restorePassphrase.length < 8) {
      showToast(t('settings.backup.errPassphrase'), 'error');
      return;
    }

    try {
      const input = document.createElement('input');
      input.type = 'file';
      input.accept = '.catb,.enc,.txt';
      input.onchange = async () => {
        const file = input.files?.[0];
        if (!file) return;
        
        showToast(t('settings.backup.importing'), 'info');
        
        const reader = new FileReader();
        reader.onload = async (event) => {
          try {
            const b64 = event.target?.result as string;
            const importedCount = await importEncryptedBackup(b64, restorePassphrase);
            showToast(t('settings.backup.restored', { count: importedCount }), 'success');
            restorePassphrase = '';
          } catch (err: any) {
            showToast(err?.message || t('settings.backup.restoreFailed'), 'error');
          }
        };
        reader.readAsText(file);
      };
      input.click();
    } catch (err: any) {
      showToast(err?.message || t('settings.backup.restoreFailed'), 'error');
    }
  }

  async function handleChangeMasterPassword(e: Event) {
    e.preventDefault();
    if (newPassword.length < MIN_VAULT_PASSWORD_LEN) {
      showToast(t('settings.security.errMin', { min: MIN_VAULT_PASSWORD_LEN }), 'error');
      return;
    }
    if (newPassword !== confirmPassword) {
      showToast(t('settings.security.errMismatch'), 'error');
      return;
    }
    isChangingPassword = true;
    try {
      await changeMasterPassword(currentPassword, newPassword);
      currentPassword = '';
      newPassword = '';
      confirmPassword = '';
      const restartNow = await confirmModal(
        t('settings.security.changedConfirm'),
        t('settings.security.changedTitle'),
        false,
        t('settings.security.restartNow'),
        t('settings.security.later')
      );
      if (restartNow) {
        await relaunch();
        return;
      }
      showToast(t('settings.security.changedToast'), 'success');
    } catch (err) {
      showToast(errorText(err), 'error');
    } finally {
      isChangingPassword = false;
    }
  }
</script>

<div class="max-w-4xl mx-auto space-y-6">
  <PageHeader
    icon={['M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z', 'M15 12a3 3 0 11-6 0 3 3 0 016 0z']}
    accent="cyan"
    title={t('settings.title')}
    subtitle={t('settings.subtitle')}
  />

  <!-- Settings Tabs -->
  <div class="border-b border-neutral-200 dark:border-neutral-800 flex gap-4 overflow-x-auto" role="tablist">
    {#each TABS as tab (tab)}
      <button
        role="tab"
        aria-selected={activeTab === tab}
        onclick={() => (activeTab = tab)}
        class="pb-3 whitespace-nowrap text-sm font-medium transition-colors border-b-2 {activeTab === tab ? 'border-sky-500 text-neutral-900 dark:text-white font-semibold' : 'border-transparent text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200'}"
      >
        {t(`settings.tabs.${tab}`)}
      </button>
    {/each}
  </div>

  {#if activeTab === 'profile'}
    <div class="{CARD} space-y-6">
      <div class="flex items-center gap-4">
        <ProfileAvatar avatar={profileAvatar} name={profileName} size={56} />
        <div class="min-w-0">
          <div class="flex items-center gap-2">
            <h2 class="text-lg font-semibold text-neutral-900 dark:text-white truncate">{profileName.trim() || profile.name}</h2>
            <span class="text-[10px] font-semibold tracking-wider px-1.5 py-0.5 rounded border border-neutral-300 dark:border-neutral-700 text-neutral-600 dark:text-neutral-400 uppercase">{t('profileMenu.planFree')}</span>
          </div>
          <p class="{MUTED} text-sm">{t('settings.profile.localNote')}</p>
        </div>
      </div>

      <form onsubmit={handleSaveProfile} class="space-y-5 max-w-md">
        <div>
          <label for="profile-name" class={LABEL}>{t('settings.profile.displayName')}</label>
          <input
            id="profile-name"
            type="text"
            maxlength="48"
            bind:value={profileName}
            placeholder="CATerm User"
            class={INPUT} />
        </div>
        <div>
          <span class="{LABEL_BASE} mb-2">{t('settings.profile.picture')}</span>
          <AvatarPicker bind:value={profileAvatar} size={40} />
        </div>
        <button
          type="submit"
          disabled={!profileDirty}
          class="px-4 py-2 bg-sky-600 hover:bg-sky-500 disabled:opacity-40 disabled:hover:bg-sky-600 text-white text-sm font-medium rounded-md transition-colors shadow-sm">
          {t('settings.profile.save')}
        </button>
      </form>

      <div class="pt-5 border-t border-neutral-200 dark:border-neutral-800 flex items-center justify-between gap-4">
        <div>
          <p class="text-sm font-medium text-neutral-900 dark:text-white">{t('settings.profile.masterPassword')}</p>
          <p class="text-xs {MUTED}">{t('settings.profile.masterPasswordBody')}</p>
        </div>
        <button
          onclick={() => (activeTab = 'security')}
          class="px-3 py-1.5 text-xs font-medium rounded-md border border-neutral-300 dark:border-neutral-700 text-neutral-700 dark:text-neutral-200 hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors">
          {t('settings.profile.changeMasterPassword')}
        </button>
      </div>
    </div>
  {:else if activeTab === 'updates'}
    <div class="{CARD} space-y-4">
      <div class="flex flex-wrap justify-between items-center gap-4">
        <div>
          <h2 class="text-lg font-semibold text-neutral-900 dark:text-white">{t('settings.updates.title')}</h2>
          <p class="{MUTED} text-sm">{t('settings.updates.current')} <span class="font-mono text-sky-600 dark:text-sky-400">v{APP_VERSION}</span></p>
        </div>
        {#if updater.status === 'available'}
          <button
            onclick={installUpdate}
            class="px-4 py-2 bg-emerald-600 hover:bg-emerald-500 text-white text-sm font-medium rounded-md transition-colors shadow-sm">
            {t('settings.updates.install')}
          </button>
        {:else}
          <button
            onclick={() => checkForUpdates()}
            disabled={updater.status === 'checking' || updater.status === 'downloading'}
            class="px-4 py-2 bg-sky-600 hover:bg-sky-500 disabled:opacity-60 text-white text-sm font-medium rounded-md transition-colors flex items-center gap-2 shadow-sm">
            {#if updater.status === 'checking'}
              <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24"><circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle><path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path></svg>
              {t('settings.updates.checking')}
            {:else}
              {t('settings.updates.check')}
            {/if}
          </button>
        {/if}
      </div>

      {#if updater.status === 'up-to-date'}
        <div class="flex items-center gap-2 p-3 rounded-md bg-emerald-500/10 border border-emerald-500/30 text-emerald-700 dark:text-emerald-400 text-sm">
          <svg class="w-4 h-4 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" /></svg>
          {t('settings.updates.latest', { version: APP_VERSION })}
        </div>
      {:else if updater.status === 'available'}
        <div class="p-3 rounded-md bg-sky-500/10 border border-sky-500/30 text-sky-700 dark:text-sky-300 text-sm">
          {t('settings.updates.available', { version: updater.version })}
          <a href={releaseNotesUrl(updater.version)} class="underline underline-offset-2 hover:text-sky-900 dark:hover:text-white">{t('settings.updates.readChangelog')}</a>
        </div>
      {:else if updater.status === 'downloading'}
        <div class="p-3 rounded-md bg-sky-500/10 border border-sky-500/30 text-sky-700 dark:text-sky-300 text-sm">
          {t('settings.updates.downloading', { version: updater.version, progress: downloadPercent === null ? '...' : ` (${downloadPercent}%)` })}
        </div>
      {:else if updater.status === 'error'}
        <div class="p-3 rounded-md bg-rose-500/10 border border-rose-500/30 text-rose-700 dark:text-rose-300 text-sm">
          {updater.error}
        </div>
      {/if}
    </div>
  {:else if activeTab === 'ai'}
    <div class="{CARD} space-y-4">
      <div>
        <h2 class="text-lg font-semibold text-neutral-900 dark:text-white">{t('settings.ai.title')}</h2>
        <p class="{MUTED} text-sm">{t('settings.ai.subtitle')}</p>
      </div>
      <AiSettingsForm />
    </div>
  {:else if activeTab === 'subscription'}
    <div class="{CARD} space-y-6">
      <div>
        <h2 class="text-lg font-semibold text-neutral-900 dark:text-white">{t('settings.subscription.title')}</h2>
        <p class="{MUTED} text-sm mt-1">{t('settings.subscription.subtitle')}</p>
      </div>

      <div class="grid grid-cols-1 lg:grid-cols-2 gap-4 items-start">
        <!-- Community: what everyone has today -->
        <div class="{SUBCARD} space-y-2">
          <div class="flex items-center gap-2">
            <span class="font-semibold text-neutral-900 dark:text-white">{t('settings.subscription.freePlan')}</span>
            <span class="text-xs bg-white dark:bg-neutral-800 text-neutral-600 dark:text-neutral-300 border border-neutral-300 dark:border-neutral-700 px-2 py-0.5 rounded">{t('settings.subscription.current')}</span>
          </div>
          <div class="text-2xl font-bold text-neutral-900 dark:text-white">$0</div>
          <p class="{MUTED} text-xs leading-relaxed">{t('settings.subscription.freeBody')}</p>
        </div>

        <!-- Pro -->
        <div class="rounded-lg border-2 border-sky-500/50 bg-sky-50/60 dark:bg-sky-950/20 p-5 space-y-4">
          <div class="flex items-center justify-between gap-2 flex-wrap">
            <span class="font-semibold text-neutral-900 dark:text-white">{t('settings.subscription.proPlan')}</span>
            <span class="text-[11px] font-bold px-2 py-0.5 rounded-full bg-rose-500 text-white">
              {t('settings.subscription.introBadge', { percent: PRO_PRICING.introDiscountPercent, months: PRO_PRICING.introMonths })}
            </span>
          </div>

          <div class="inline-flex p-0.5 rounded-lg border border-neutral-200 dark:border-neutral-700 bg-white dark:bg-neutral-900 text-xs" role="group">
            {#each ['monthly', 'yearly'] as const as interval (interval)}
              <button
                type="button"
                aria-pressed={billing === interval}
                onclick={() => (billing = interval)}
                class="px-3 py-1 rounded-md font-medium transition-colors {billing === interval ? 'bg-sky-600 text-white' : 'text-neutral-600 dark:text-neutral-300 hover:text-neutral-900 dark:hover:text-white'}"
              >
                {t(`settings.subscription.${interval}`)}
                {#if interval === 'yearly'}
                  <span class="ml-1 text-[10px] font-semibold {billing === 'yearly' ? 'text-sky-100' : 'text-emerald-600 dark:text-emerald-400'}">{t('settings.subscription.saveYearly', { percent: PRO_PRICING.yearlyDiscountPercent })}</span>
                {/if}
              </button>
            {/each}
          </div>

          <div>
            <div class="flex items-baseline gap-2">
              <span class="text-sm text-neutral-400 line-through">{formatUsd(PRO_PRICING[billing].list)}</span>
              <span class="text-3xl font-bold text-neutral-900 dark:text-white">{formatUsd(PRO_PRICING[billing].intro)}</span>
              <span class="text-sm text-neutral-500">{billing === 'monthly' ? t('settings.subscription.perMonth') : t('settings.subscription.perYear')}</span>
            </div>
            <p class="text-xs {MUTED} mt-1">
              {#if billing === 'monthly'}
                {t('settings.subscription.monthlyThen', { months: PRO_PRICING.introMonths, price: formatUsd(PRO_PRICING.monthly.list) })}
              {:else}
                {t('settings.subscription.yearlyThen', { perMonth: formatUsd(PRO_PRICING.yearly.intro / 12), price: formatUsd(PRO_PRICING.yearly.list) })}
              {/if}
            </p>
          </div>

          <ul class="space-y-1.5 text-xs text-neutral-700 dark:text-neutral-300">
            {#each proFeatures as feature (feature)}
              <li class="flex items-start gap-2">
                <svg class="w-4 h-4 text-emerald-600 dark:text-emerald-400 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" /></svg>
                <span>{feature}</span>
              </li>
            {/each}
          </ul>

          <div class="flex flex-col sm:flex-row gap-2">
            <button disabled class="flex-1 px-4 py-2 rounded-md text-sm font-semibold border border-sky-500/40 text-sky-700/70 dark:text-sky-300/70 cursor-not-allowed" title={t('settings.subscription.comingSoon')}>
              {t('settings.subscription.startTrial', { days: PRO_PRICING.trialDays })}
              <span class="block text-[10px] font-normal">{t('settings.subscription.comingSoon')}</span>
            </button>
            <button
              type="button"
              onclick={() => openExternalUrl(PRICING_URL)}
              class="flex-1 inline-flex items-center justify-center gap-1.5 px-4 py-2 rounded-md text-sm font-semibold bg-sky-600 hover:bg-sky-500 text-white shadow-sm transition-colors"
            >
              {t('settings.subscription.subscribe')}
              <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" /></svg>
            </button>
          </div>
          <p class="text-[11px] text-neutral-500">{t('settings.subscription.trialNote')}</p>
        </div>
      </div>

      <div class="pt-4 border-t border-neutral-200 dark:border-neutral-800/80 flex items-center justify-between gap-4">
        <div>
          <p class="text-xs text-neutral-800 dark:text-neutral-300 font-medium">{t('settings.subscription.supportTitle')}</p>
          <p class="text-[11px] text-neutral-500">{t('settings.subscription.supportBody')}</p>
        </div>
        <a
          href="https://paypal.me/cecepazhar"
          target="_blank"
          rel="noopener"
          class="inline-flex items-center gap-2 px-4 py-2 bg-[#0070ba] hover:bg-[#005ea6] text-white text-xs font-semibold rounded-lg transition-colors shrink-0"
        >
          <span>{t('contribution.donateVia')}</span>
        </a>
      </div>
    </div>
  {:else if activeTab === 'security'}
    <div class="{CARD} space-y-4">
      <h2 class="text-lg font-semibold text-neutral-900 dark:text-white">{t('settings.security.title')}</h2>
      <p class="{MUTED} text-sm">{t('settings.security.subtitle')}</p>

      <form onsubmit={handleChangeMasterPassword} class="max-w-md space-y-4 pt-2">
        <p class="text-xs {MUTED}">{t('settings.security.rekeyNote')}</p>
        <div>
          <label for="current-pass" class={LABEL}>{t('settings.security.current')}</label>
          <input
            id="current-pass"
            type={showPasswords ? 'text' : 'password'}
            required
            autocomplete="current-password"
            bind:value={currentPassword}
            class={INPUT} />
        </div>
        <div>
          <label for="new-pass" class={LABEL}>{t('settings.security.new', { min: MIN_VAULT_PASSWORD_LEN })}</label>
          <input
            id="new-pass"
            type={showPasswords ? 'text' : 'password'}
            required
            minlength={MIN_VAULT_PASSWORD_LEN}
            autocomplete="new-password"
            bind:value={newPassword}
            class={INPUT} />
        </div>
        <div>
          <label for="confirm-pass" class={LABEL}>{t('settings.security.confirm')}</label>
          <input
            id="confirm-pass"
            type={showPasswords ? 'text' : 'password'}
            required
            minlength={MIN_VAULT_PASSWORD_LEN}
            autocomplete="new-password"
            bind:value={confirmPassword}
            class={INPUT} />
        </div>
        <label class="flex items-center gap-2 text-xs {MUTED}">
          <input type="checkbox" bind:checked={showPasswords} class="rounded" />
          {t('settings.security.showPasswords')}
        </label>
        <button
          type="submit"
          disabled={isChangingPassword}
          class="px-4 py-2 bg-emerald-600 hover:bg-emerald-500 disabled:opacity-60 text-white text-sm font-medium rounded-md transition-colors">
          {isChangingPassword ? t('settings.security.reencrypting') : t('settings.security.change')}
        </button>
      </form>

      <!-- Audit Trail Settings -->
      <div class="border-t border-neutral-200 dark:border-neutral-800 pt-6 space-y-3">
        <div>
          <h3 class="text-base font-semibold text-neutral-900 dark:text-white">{t('settings.audit.title')}</h3>
          <p class="{MUTED} text-xs mt-1">{t('settings.audit.body')}</p>
        </div>
        <form onsubmit={handleSaveAuditSettings} class="flex items-end gap-4 max-w-sm">
          <div class="flex-1">
            <label for="max-audit" class={LABEL}>{t('settings.audit.maxRecords')}</label>
            <input
              id="max-audit"
              type="number"
              min="100"
              max="50000"
              step="100"
              bind:value={maxAuditRecords}
              class={INPUT}
            />
          </div>
          <button
            type="submit"
            class="px-4 py-2 bg-sky-600 hover:bg-sky-500 text-white text-sm font-medium rounded-md transition-colors whitespace-nowrap"
          >
            {t('common.save')}
          </button>
        </form>
      </div>
    </div>
  {:else if activeTab === 'backup'}
    <div class="{CARD} space-y-8">
      <div>
        <h2 class="text-lg font-semibold text-neutral-900 dark:text-white">{t('settings.backup.title')}</h2>
        <p class="{MUTED} text-sm mt-1">{t('settings.backup.subtitle')}</p>
      </div>

      <div class="space-y-4">
        <h3 class="text-base font-semibold text-neutral-900 dark:text-white">{t('settings.backup.exportTitle')}</h3>
        <p class="text-xs {MUTED}">{t('settings.backup.exportBody')}</p>
        <form onsubmit={handleExport} class="max-w-md space-y-4 pt-2">
          <div>
            <label for="backup-pass" class={LABEL}>{t('settings.backup.exportPassphrase')}</label>
            <div class="relative">
              <input
                id="backup-pass"
                type={showBackupPassphrase ? 'text' : 'password'}
                minlength="8"
                required
                bind:value={backupPassphrase}
                class="{INPUT} pr-10" />
              <button
                type="button"
                onclick={() => (showBackupPassphrase = !showBackupPassphrase)}
                class={EYE_BUTTON}
                aria-label={showBackupPassphrase ? t('settings.backup.hidePassphrase') : t('settings.backup.showPassphrase')}
                title={showBackupPassphrase ? t('settings.backup.hidePassphrase') : t('settings.backup.showPassphrase')}
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
            <p class="text-xs {backupMsgKind === 'success' ? 'text-emerald-600 dark:text-emerald-500' : 'text-red-600 dark:text-red-500'}">{backupMsg}</p>
          {/if}
          <button type="submit" class="px-4 py-2 bg-sky-600 hover:bg-sky-500 text-white text-sm font-medium rounded-md transition-colors">
            {t('settings.backup.exportButton')}
          </button>
        </form>
      </div>

      <hr class="border-neutral-200 dark:border-neutral-800" />

      <div class="space-y-4">
        <h3 class="text-base font-semibold text-neutral-900 dark:text-white">{t('settings.backup.restoreTitle')}</h3>
        <p class="text-xs {MUTED}">{t('settings.backup.restoreBody')}</p>
        <form onsubmit={handleImport} class="max-w-md space-y-4 pt-2">
          <div>
            <label for="restore-pass" class={LABEL}>{t('settings.backup.restorePassphrase')}</label>
            <div class="relative">
              <input
                id="restore-pass"
                type={showRestorePassphrase ? 'text' : 'password'}
                required
                bind:value={restorePassphrase}
                class="{INPUT} pr-10" />
              <button
                type="button"
                onclick={() => (showRestorePassphrase = !showRestorePassphrase)}
                class={EYE_BUTTON}
                aria-label={showRestorePassphrase ? t('settings.backup.hidePassphrase') : t('settings.backup.showPassphrase')}
                title={showRestorePassphrase ? t('settings.backup.hidePassphrase') : t('settings.backup.showPassphrase')}
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
            <p class="text-xs {restoreMsgKind === 'success' ? 'text-emerald-600 dark:text-emerald-500' : 'text-red-600 dark:text-red-500'}">{restoreMsg}</p>
          {/if}
          <button type="submit" class="px-4 py-2 bg-amber-600 hover:bg-amber-500 text-white text-sm font-medium rounded-md transition-colors">
            {t('settings.backup.restoreButton')}
          </button>
        </form>
      </div>
    </div>
  {:else if activeTab === 'performance'}
    <div class="{CARD} space-y-6">
      <div>
        <h2 class="text-lg font-semibold text-neutral-900 dark:text-white">{t('settings.performance.title')}</h2>
        <p class="{MUTED} text-sm mt-1">{t('settings.performance.subtitle')}</p>
      </div>

      <div class="{SUBCARD} flex items-start justify-between gap-4">
        <div class="min-w-0">
          <p class="text-sm font-medium text-neutral-900 dark:text-white">{t('settings.performance.backgroundTitle')}</p>
          <p class="text-xs {MUTED} mt-1 leading-relaxed">{t('settings.performance.backgroundBody')}</p>
        </div>
        <span class="shrink-0 text-[11px] font-semibold px-2 py-0.5 rounded-full bg-emerald-500/10 text-emerald-700 dark:text-emerald-400 border border-emerald-500/30">{t('settings.performance.alwaysOn')}</span>
      </div>

      <div class="{SUBCARD} flex items-start justify-between gap-4">
        <div class="min-w-0">
          <p class="text-sm font-medium text-neutral-900 dark:text-white">{t('settings.performance.gpuTitle')}</p>
          <p class="text-xs {MUTED} mt-1 leading-relaxed">{t('settings.performance.gpuBody')}</p>
          <p class="text-[11px] text-neutral-500 mt-2">{t('settings.performance.recommended')} · {t('settings.performance.restartNote')}</p>
        </div>
        <button
          type="button"
          role="switch"
          aria-checked={gpuAcceleration}
          aria-label={t('settings.performance.gpuTitle')}
          disabled={isSavingPerformance}
          onclick={toggleGpuAcceleration}
          class="relative shrink-0 w-11 h-6 rounded-full transition-colors disabled:opacity-50 {gpuAcceleration ? 'bg-sky-600' : 'bg-neutral-300 dark:bg-neutral-700'}"
        >
          <span class="absolute top-0.5 left-0.5 w-5 h-5 rounded-full bg-white shadow transition-transform {gpuAcceleration ? 'translate-x-5' : ''}"></span>
        </button>
      </div>
    </div>
  {:else if activeTab === 'shortcuts'}
    <div class="{CARD} space-y-6">
      <div>
        <h2 class="text-lg font-semibold text-neutral-900 dark:text-white">{t('settings.shortcuts.title')}</h2>
        <p class="{MUTED} text-sm mt-1">{t('settings.shortcuts.subtitle')}</p>
      </div>

      <div class="space-y-4">
        {#each SHORTCUT_GROUPS as group, gi (group.title)}
          <h3 class="text-sm font-semibold text-neutral-600 dark:text-neutral-300 uppercase tracking-wider {gi > 0 ? 'pt-2' : ''}">{t(`settings.shortcuts.${group.title}`)}</h3>
          <div class="space-y-2">
            {#each group.items as item (item.keys)}
              <div class="flex justify-between items-center p-3 bg-neutral-50 dark:bg-neutral-950 border border-neutral-200 dark:border-neutral-800 rounded-md">
                <span class="text-sm text-neutral-800 dark:text-neutral-200">{t(`settings.shortcuts.${item.label}`, item.vars)}</span>
                <kbd class="px-2 py-1 bg-white dark:bg-neutral-800 border border-neutral-300 dark:border-neutral-700 text-neutral-700 dark:text-neutral-300 text-xs rounded font-mono">{item.keys}</kbd>
              </div>
            {/each}
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>
