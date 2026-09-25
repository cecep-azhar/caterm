<script lang="ts">
  import { t } from '$lib/i18n/index.svelte';
  import { SPONSOR_TIERS, SPONSORS, type SponsorTierId } from '$lib/data/sponsors';
  import { GITHUB_SPONSORS_URL } from '$lib/appInfo';

  const TIER_LABEL_KEY: Record<SponsorTierId, string> = {
    platinum: 'sponsorWall.tierPlatinum',
    gold: 'sponsorWall.tierGold',
    silver: 'sponsorWall.tierSilver',
    contributor: 'sponsorWall.tierContributor'
  };

  const bySponsorTier = (tier: SponsorTierId) => SPONSORS.filter((s) => s.tier === tier);

  function initialsOf(name: string): string {
    return name.trim().slice(0, 1).toUpperCase() || '?';
  }
</script>

<div class="p-6 rounded-2xl bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 space-y-5">
  <div class="space-y-1">
    <h3 class="text-base font-semibold text-neutral-900 dark:text-white">{t('sponsorWall.title')}</h3>
    <p class="text-xs text-neutral-500 dark:text-neutral-400">{t('sponsorWall.subtitle')}</p>
  </div>

  <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
    {#each SPONSOR_TIERS as tier (tier.id)}
      {@const sponsors = bySponsorTier(tier.id)}
      <div class="p-4 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-neutral-50/60 dark:bg-neutral-950/40 space-y-3">
        <div class="flex items-center gap-2">
          <span
            class="w-2.5 h-2.5 rounded-full shrink-0"
            style="background: linear-gradient(135deg, {tier.from}, {tier.to});"
          ></span>
          <span class="text-sm font-semibold text-neutral-900 dark:text-white">{t(TIER_LABEL_KEY[tier.id])}</span>
          <span class="text-[11px] text-neutral-400 dark:text-neutral-500 ml-auto">
            {tier.minUsd ? t('sponsorWall.minAmount', { amount: tier.minUsd }) : t('sponsorWall.anyAmount')}
          </span>
        </div>

        {#if sponsors.length > 0}
          <ul class="flex flex-wrap gap-2">
            {#each sponsors as sponsor (sponsor.name)}
              <li>
                <a
                  href={sponsor.url ?? GITHUB_SPONSORS_URL}
                  target="_blank"
                  rel="noreferrer"
                  class="flex items-center gap-1.5 pl-1 pr-2.5 py-1 rounded-full bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 hover:border-sky-500/50 transition-colors text-xs font-medium text-neutral-700 dark:text-neutral-300"
                >
                  <span
                    class="w-5 h-5 rounded-full flex items-center justify-center text-[10px] font-bold text-white shrink-0"
                    style="background: linear-gradient(135deg, {tier.from}, {tier.to});"
                  >
                    {initialsOf(sponsor.name)}
                  </span>
                  {sponsor.name}
                </a>
              </li>
            {/each}
          </ul>
        {:else}
          <p class="text-xs text-neutral-400 dark:text-neutral-500 italic">{t('sponsorWall.emptyTier')}</p>
        {/if}
      </div>
    {/each}
  </div>

  <a
    href={GITHUB_SPONSORS_URL}
    target="_blank"
    rel="noreferrer"
    class="inline-flex items-center gap-2 px-4 py-2.5 rounded-xl bg-neutral-900 dark:bg-white text-white dark:text-neutral-900 text-sm font-semibold hover:opacity-90 transition-opacity"
  >
    <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24"><path d="M12 .5C5.73.5.5 5.73.5 12c0 5.08 3.29 9.39 7.86 10.91.57.1.79-.25.79-.55v-1.94c-3.2.7-3.88-1.54-3.88-1.54-.52-1.34-1.28-1.7-1.28-1.7-1.05-.72.08-.7.08-.7 1.16.08 1.78 1.19 1.78 1.19 1.03 1.77 2.7 1.26 3.36.96.1-.75.4-1.26.73-1.55-2.56-.29-5.26-1.28-5.26-5.71 0-1.26.45-2.29 1.19-3.09-.12-.29-.52-1.46.11-3.05 0 0 .97-.31 3.18 1.18a11.1 11.1 0 015.8 0c2.21-1.49 3.18-1.18 3.18-1.18.63 1.59.23 2.76.11 3.05.74.8 1.19 1.83 1.19 3.09 0 4.44-2.71 5.42-5.29 5.7.42.36.78 1.07.78 2.16v3.2c0 .31.21.66.8.55A10.52 10.52 0 0023.5 12C23.5 5.73 18.27.5 12 .5z"/></svg>
    {t('sponsorWall.cta')}
  </a>
</div>
