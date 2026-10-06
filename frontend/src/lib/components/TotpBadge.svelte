<script lang="ts">
  import { showToast } from '$lib/stores/uiNotifications.svelte';
  import { copyText } from '$lib/utils/clipboard';

  let {
    token = '000000',
    remainingSeconds = 30,
    totalPeriod = 30,
    label = '',
    issuer = '',
    onCopy
  }: {
    token: string;
    remainingSeconds: number;
    totalPeriod?: number;
    label?: string;
    issuer?: string;
    onCopy?: () => void;
  } = $props();

  let copied = $state(false);

  const formattedToken = $derived.by(() => {
    const clean = token.replace(/\s+/g, '');
    if (clean.length === 6) {
      return `${clean.slice(0, 3)} ${clean.slice(3)}`;
    }
    return token;
  });

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
</script>

<div
  class="group relative inline-flex items-center justify-between gap-3 p-3 bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-xl hover:border-sky-500/50 transition-all shadow-xs"
>
  <div class="min-w-0 flex-1">
    {#if issuer || label}
      <div class="flex items-center gap-1.5 truncate mb-1">
        {#if issuer}
          <span class="text-[10px] font-bold uppercase tracking-wider px-1.5 py-0.2 rounded bg-sky-500/15 text-sky-700 dark:text-sky-300 border border-sky-500/30">
            {issuer}
          </span>
        {/if}
        <span class="text-xs font-semibold text-neutral-900 dark:text-white truncate">
          {label}
        </span>
      </div>
    {/if}
    <div class="flex items-center gap-2">
      <span class="font-mono text-xl sm:text-2xl font-bold tracking-wider text-neutral-900 dark:text-white">
        {formattedToken}
      </span>
    </div>
  </div>

  <div class="flex items-center gap-2 shrink-0">
    <!-- Circular countdown timer -->
    <div class="relative w-8 h-8 flex items-center justify-center">
      <svg class="w-8 h-8 -rotate-90 transform" viewBox="0 0 36 36">
        <!-- Background ring -->
        <path
          class="text-neutral-200 dark:text-neutral-800"
          stroke="currentColor"
          stroke-width="3"
          fill="none"
          d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831"
        />
        <!-- Progress ring -->
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
      <span class="absolute text-[10px] font-mono font-bold {timerColor}">
        {remainingSeconds}
      </span>
    </div>

    <!-- 1-Click Copy Button -->
    <button
      type="button"
      onclick={handleCopy}
      class="p-2 rounded-lg bg-neutral-100 dark:bg-neutral-800 hover:bg-sky-500 hover:text-white text-neutral-600 dark:text-neutral-300 transition-colors"
      title="Salin Kode 2FA"
      aria-label="Salin Kode 2FA"
    >
      {#if copied}
        <!-- Checkmark -->
        <svg class="w-4 h-4 text-emerald-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M5 13l4 4L19 7" />
        </svg>
      {:else}
        <!-- Copy Icon -->
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
        </svg>
      {/if}
    </button>
  </div>
</div>
