<script lang="ts">
  import type { Snippet } from 'svelte';
  import { getPro } from '$lib/stores/pro.svelte';
  import { openExternalUrl } from '$lib/utils/url';

  interface Props {
    title?: string;
    description?: string;
    children?: Snippet;
  }

  let {
    title = 'Fitur Pro Eksklusif',
    description = 'Fitur ini membutuhkan lisensi aktif CATerm Pro atau tim berbayar.',
    children
  }: Props = $props();

  const pro = getPro();

  const pricingUrl = 'https://caterm.fathforce.com/pricing';

  function handleUpgrade() {
    openExternalUrl(pricingUrl);
  }
</script>

{#if pro.isPro}
  {@render children?.()}
{:else}
  <div class="relative w-full h-full min-h-[220px] rounded-xl overflow-hidden border border-neutral-200/80 dark:border-neutral-800 bg-neutral-50/50 dark:bg-neutral-900/50">
    <!-- Blurred Background Content Preview -->
    <div class="filter blur-md pointer-events-none select-none opacity-30 p-4 w-full h-full" aria-hidden="true">
      {@render children?.()}
    </div>

    <!-- Dark Overlay with Gold Lock Guard -->
    <div class="absolute inset-0 z-20 flex flex-col items-center justify-center p-6 text-center bg-black/60 backdrop-blur-md">
      <div class="w-14 h-14 rounded-2xl bg-amber-500/10 border border-amber-500/30 flex items-center justify-center text-2xl shadow-lg mb-3 shadow-amber-500/5 animate-pulse">
        <span role="img" aria-label="Locked">🔒</span>
      </div>

      <h3 class="text-base font-bold text-white tracking-tight mb-1">
        {title}
      </h3>

      <p class="text-xs text-neutral-300 max-w-sm mb-5 leading-relaxed">
        {description}
      </p>

      <div class="flex items-center gap-3">
        <button
          type="button"
          onclick={handleUpgrade}
          class="px-5 py-2 rounded-lg bg-gradient-to-r from-amber-500 to-amber-600 hover:from-amber-400 hover:to-amber-500 text-neutral-950 font-semibold text-xs tracking-wide shadow-md transition-all duration-150 transform hover:scale-[1.02] active:scale-[0.98] cursor-pointer"
        >
          Aktivasi Pro ($10/bulan)
        </button>
        <a
          href="/settings"
          class="px-4 py-2 rounded-lg border border-neutral-600 hover:border-neutral-400 text-neutral-300 hover:text-white text-xs font-medium transition-colors"
        >
          Status Akun
        </a>
      </div>

      <p class="mt-4 text-[11px] text-neutral-400">
        Termasuk 7 hari uji coba gratis · Tanpa kartu kredit
      </p>
    </div>
  </div>
{/if}
