<script lang="ts">
  import { openExternalUrl } from '$lib/utils/url';
  let copied = $state(false);

  function copyPaypal() {
    navigator.clipboard.writeText('https://paypal.me/cecepazhar');
    copied = true;
    setTimeout(() => { copied = false; }, 2000);
  }
</script>

<div class="max-w-4xl mx-auto space-y-8 pb-12">
  <!-- Header -->
  <div class="border-b border-neutral-200 dark:border-neutral-800 pb-6 mb-6">
    <div class="flex items-center gap-3 mb-2">
      <div class="p-2 rounded-xl bg-rose-500/10 text-rose-500">
        <svg class="w-6 h-6" fill="currentColor" viewBox="0 0 24 24">
          <path d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z"/>
        </svg>
      </div>
      <div>
        <h1 class="text-2xl font-bold text-neutral-900 dark:text-white tracking-tight">Contribution &amp; Support CATerm</h1>
        <p class="text-sm text-neutral-500 dark:text-neutral-400">Support the independent development of CATerm — 100% Free, Local-First, Zero-Knowledge.</p>
      </div>
    </div>
  </div>

  <!-- Main Support Banner -->
  <div class="p-6 rounded-2xl bg-gradient-to-br from-sky-50 to-white dark:from-neutral-900 dark:to-neutral-950 border border-neutral-200 dark:border-neutral-800 shadow-xl space-y-6 text-neutral-900 dark:text-white">
    <div class="space-y-2">
      <span class="text-xs uppercase tracking-wider font-semibold text-sky-600 dark:text-sky-400">Direct Donation</span>
      <h2 class="text-xl font-bold">Help Keep CATerm Independent, Fast, and Free of Ads &amp; Paywalls</h2>
      <p class="text-neutral-600 dark:text-neutral-300 text-sm leading-relaxed">
        CATerm is engineered with native Rust and modern Svelte without hidden analytics, tracking servers, or subscription paywalls for essential terminal &amp; SFTP workflows. Every donation directly supports open-source maintenance, multi-platform builds, and upcoming innovations like local AI agents.
      </p>
    </div>

    <!-- PayPal Action -->
    <div class="flex flex-col sm:flex-row items-start sm:items-center gap-4 pt-2">
      <a
        href="https://paypal.me/cecepazhar"
        target="_blank"
        rel="noopener"
        class="inline-flex items-center gap-2.5 px-6 py-3 bg-[#0070ba] hover:bg-[#005ea6] text-white font-semibold rounded-xl text-sm shadow-lg shadow-sky-900/30 transition-all hover:scale-[1.02]"
      >
        <svg class="w-4 h-4 fill-current" viewBox="0 0 24 24">
          <path d="M20.067 8.478c.492.88.556 2.014.3 3.327-.777 3.99-3.42 6.07-7.857 6.07H9.288a.936.936 0 0 1-.925-.797L6.82 7.02a.936.936 0 0 1 .925-1.077h5.08c2.81 0 4.88.428 5.894 1.488.583.61.94 1.34 1.348 2.047zm-5.074 1.09c-.588-.616-1.78-.865-3.393-.865H8.847l-1.09 6.883h2.368c2.87 0 4.542-1.348 5.04-3.905.215-1.106.143-1.63-.172-2.113z"/>
        </svg>
        <span>Donate via PayPal (paypal.me/cecepazhar)</span>
      </a>

      <button
        onclick={copyPaypal}
        class="px-4 py-3 bg-neutral-100 hover:bg-neutral-200 dark:bg-neutral-800 dark:hover:bg-neutral-700 text-neutral-700 dark:text-neutral-300 rounded-xl text-sm font-medium border border-neutral-300 dark:border-neutral-700 transition-colors"
      >
        {copied ? '✓ Copied Link' : 'Copy PayPal URL'}
      </button>
    </div>
  </div>

  <!-- Other Contribution Ways -->
  <div class="space-y-4 pt-2">
    <h3 class="text-base font-semibold text-neutral-900 dark:text-white">Other Ways to Contribute</h3>
    <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
      <a
        href="https://github.com/cecep-azhar/caterm"
        target="_blank"
        rel="noreferrer"
        class="p-5 rounded-xl bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 space-y-2 hover:border-amber-500/50 hover:bg-neutral-50 dark:hover:bg-neutral-850 transition-all block group"
      >
        <div class="text-amber-500 flex items-center justify-between">
          <svg class="w-5 h-5 fill-current" viewBox="0 0 24 24"><path d="M12 17.27L18.18 21l-1.64-7.03L22 9.24l-7.19-.61L12 2 9.19 8.63 2 9.24l5.46 4.73L5.82 21z"/></svg>
          <span class="text-xs text-neutral-400 group-hover:text-amber-400 font-mono flex items-center gap-1">Visit ↗</span>
        </div>
        <h4 class="font-bold text-neutral-900 dark:text-white text-sm">Star on GitHub</h4>
        <p class="text-xs text-neutral-500 dark:text-neutral-400 leading-relaxed">Star the official repository to help more engineers discover CATerm.</p>
      </a>

      <a
        href="https://github.com/cecep-azhar/caterm/issues"
        target="_blank"
        rel="noreferrer"
        class="p-5 rounded-xl bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 space-y-2 hover:border-sky-500/50 hover:bg-neutral-50 dark:hover:bg-neutral-850 transition-all block group"
      >
        <div class="text-sky-500 flex items-center justify-between">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"/></svg>
          <span class="text-xs text-neutral-400 group-hover:text-sky-400 font-mono flex items-center gap-1">Open ↗</span>
        </div>
        <h4 class="font-bold text-neutral-900 dark:text-white text-sm">Report Issues</h4>
        <p class="text-xs text-neutral-500 dark:text-neutral-400 leading-relaxed">Found a bug or have a feature request? Open an issue on GitHub for continuous improvement.</p>
      </a>

      <div class="p-5 rounded-xl bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 space-y-2">
        <div class="text-emerald-500">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8.684 13.342C8.886 12.938 9 12.482 9 12c0-.482-.114-.938-.316-1.342m0 2.684a3 3 0 110-2.684m0 2.684l6.632 3.316m-6.632-6l6.632-3.316m0 0a3 3 0 105.367-2.684 3 3 0 00-5.367 2.684zm0 9.316a3 3 0 105.368 2.684 3 3 0 00-5.368-2.684z"/></svg>
        </div>
        <h4 class="font-bold text-neutral-900 dark:text-white text-sm">Share with Community</h4>
        <p class="text-xs text-neutral-500 dark:text-neutral-400 leading-relaxed">Recommend CATerm to sysadmins, developers, and cloud engineers on your team.</p>
      </div>

      <a
        href="https://caterm.fathforce.com"
        target="_blank"
        rel="noreferrer"
        class="p-5 rounded-xl bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 space-y-2 hover:border-violet-500/50 hover:bg-neutral-50 dark:hover:bg-neutral-850 transition-all block group md:col-span-3"
      >
        <div class="text-violet-500 flex items-center justify-between">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9"/></svg>
          <span class="text-xs text-neutral-400 group-hover:text-violet-400 font-mono flex items-center gap-1">Visit ↗</span>
        </div>
        <h4 class="font-bold text-neutral-900 dark:text-white text-sm">Official Website</h4>
        <p class="text-xs text-neutral-500 dark:text-neutral-400 leading-relaxed">Visit https://caterm.fathforce.com for documentation, release notes, and updates.</p>
      </a>
    </div>
  </div>
</div>
