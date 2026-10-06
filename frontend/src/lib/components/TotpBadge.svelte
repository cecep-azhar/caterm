<script lang="ts">
  import { showToast } from '$lib/stores/uiNotifications.svelte';
  import { copyText } from '$lib/utils/clipboard';

  let {
    id = '',
    token = '000000',
    remainingSeconds = 30,
    totalPeriod = 30,
    label = '',
    issuer = '',
    onCopy,
    onEdit,
    onDelete
  }: {
    id?: string;
    token: string;
    remainingSeconds: number;
    totalPeriod?: number;
    label?: string;
    issuer?: string;
    onCopy?: () => void;
    onEdit?: () => void;
    onDelete?: () => void;
  } = $props();

  let copied = $state(false);
  let showCode = $state(true);

  const formattedToken = $derived.by(() => {
    const clean = token.replace(/\s+/g, '');
    if (clean.length === 6) {
      return `${clean.slice(0, 3)} ${clean.slice(3)}`;
    }
    return token;
  });

  const displayToken = $derived(
    showCode ? formattedToken : '••• •••'
  );

  const progressPercent = $derived(
    Math.max(0, Math.min(100, (remainingSeconds / totalPeriod) * 100))
  );

  const strokeDashoffset = $derived(
    100 - progressPercent
  );

  const timerColor = $derived(
    remainingSeconds <= 5
      ? 'text-rose-500'
      : remainingSeconds <= 10
        ? 'text-amber-500'
        : 'text-sky-500 dark:text-sky-400'
  );

  async function handleCopy(e: MouseEvent) {
    e.stopPropagation();
    const clean = token.replace(/\s+/g, '');
    await copyText(clean);
    copied = true;
    showToast(`2FA Token ${formattedToken} tersalin!`, 'success');
    onCopy?.();
    setTimeout(() => {
      copied = false;
    }, 2000);
  }

  function toggleShowCode(e: MouseEvent) {
    e.stopPropagation();
    showCode = !showCode;
  }
</script>

<div
  class="group rounded-xl border p-4 transition-colors bg-white dark:bg-neutral-900/40 border-neutral-200 dark:border-neutral-800 hover:border-neutral-300 dark:hover:border-neutral-700 flex flex-col justify-between"
>
  <!-- Identity / Header similar to HostCard -->
  <div class="flex items-start gap-3">
    <span class="w-10 h-10 rounded-lg bg-sky-500/10 text-sky-600 dark:text-sky-400 flex items-center justify-center shrink-0 border border-sky-500/20">
      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
      </svg>
    </span>

    <div class="min-w-0 flex-1">
      <div class="flex items-center gap-2">
        <p class="font-semibold text-neutral-900 dark:text-white truncate text-sm">{label || 'Authenticator Token'}</p>
        {#if issuer}
          <span class="inline-flex items-center px-1.5 py-0.2 rounded text-[10px] font-bold uppercase tracking-wider bg-sky-500/15 text-sky-700 dark:text-sky-300 border border-sky-500/30 shrink-0">
            {issuer}
          </span>
        {/if}
      </div>

      <div class="flex items-center gap-2 min-w-0 mt-1">
        <span class="font-mono text-xl sm:text-2xl font-bold tracking-wider text-neutral-900 dark:text-white select-all">
          {displayToken}
        </span>
        <button
          type="button"
          onclick={toggleShowCode}
          class="p-0.5 rounded text-neutral-400 hover:text-neutral-700 dark:hover:text-neutral-200 hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors shrink-0"
          title={showCode ? 'Sembunyikan Kode 2FA' : 'Tampilkan Kode 2FA'}
          aria-label={showCode ? 'Sembunyikan Kode 2FA' : 'Tampilkan Kode 2FA'}
        >
          {#if showCode}
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l18 18" />
            </svg>
          {:else}
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
            </svg>
          {/if}
        </button>
      </div>
    </div>

    <!-- Circular Timer Progress -->
    <div class="relative w-9 h-9 flex items-center justify-center shrink-0">
      <svg class="w-9 h-9 -rotate-90 transform" viewBox="0 0 36 36">
        <path
          class="text-neutral-200 dark:text-neutral-800"
          stroke="currentColor"
          stroke-width="3"
          fill="none"
          d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831"
        />
        <path
          class="{timerColor} transition-all duration-1000 ease-linear"
          stroke="currentColor"
          stroke-width="3"
          stroke-dasharray="100, 100"
          stroke-dashoffset={strokeDashoffset}
          stroke-linecap="round"
          fill="none"
          d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831"
        />
      </svg>
      <span class="absolute text-[11px] font-mono font-bold {timerColor}">
        {remainingSeconds}
      </span>
    </div>
  </div>

  <!-- Bottom Actions Footer, perfectly matching HostCard -->
  <div class="mt-3 pt-3 border-t border-neutral-100 dark:border-neutral-800/80 flex items-center gap-1.5">
    <span class="text-[11px] text-neutral-400 dark:text-neutral-500 truncate">
      Zero-Knowledge AES-256-GCM · RFC 6238 TOTP
    </span>

    {#if onEdit}
      <button
        type="button"
        onclick={onEdit}
        class="ml-auto w-8 h-8 flex items-center justify-center rounded-lg text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors"
        title="Ubah Kunci 2FA"
        aria-label="Ubah Kunci 2FA"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M16.9 4.1a2.1 2.1 0 013 3L8.5 18.5 4 20l1.5-4.5z" /></svg>
      </button>
    {/if}

    {#if onDelete}
      <button
        type="button"
        onclick={onDelete}
        class="w-8 h-8 flex items-center justify-center rounded-lg text-rose-500 hover:bg-rose-50 dark:hover:bg-rose-950/30 transition-colors"
        title="Hapus Kunci 2FA"
        aria-label="Hapus Kunci 2FA"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" /></svg>
      </button>
    {/if}

    <button
      type="button"
      onclick={handleCopy}
      class="h-8 px-3 flex items-center gap-1.5 rounded-lg bg-sky-600 hover:bg-sky-500 text-white text-xs font-semibold transition-colors"
      title="Salin Kode 2FA"
      aria-label="Salin Kode 2FA"
    >
      {#if copied}
        <svg class="w-3.5 h-3.5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M5 13l4 4L19 7" />
        </svg>
        <span>Tersalin!</span>
      {:else}
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
        </svg>
        <span>Salin Kode</span>
      {/if}
    </button>
  </div>
</div>
