<script lang="ts">
  let copied = $state(false);

  function copyPaypal() {
    navigator.clipboard.writeText('https://paypal.me/cecepazhar');
    copied = true;
    setTimeout(() => { copied = false; }, 2000);
  }
</script>

<div class="max-w-4xl mx-auto space-y-8 pb-12">
  <!-- Header -->
  <div class="border-b border-neutral-200 dark:border-neutral-800 pb-6">
    <div class="flex items-center gap-3 mb-2">
      <div class="p-2 rounded-xl bg-rose-500/10 text-rose-500">
        <svg class="w-6 h-6" fill="currentColor" viewBox="0 0 24 24">
          <path d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z"/>
        </svg>
      </div>
      <div>
        <h1 class="text-2xl font-bold text-neutral-900 dark:text-white tracking-tight">Contribution &amp; Support CATerm</h1>
        <p class="text-sm text-neutral-500 dark:text-neutral-400">Dukungan untuk pengembangan mandiri CATerm — 100% Gratis, Local-First, Zero-Knowledge.</p>
      </div>
    </div>
  </div>

  <!-- Main Support Banner -->
  <div class="p-6 rounded-2xl bg-gradient-to-br from-neutral-900 to-neutral-950 border border-neutral-800 shadow-xl space-y-6 text-white">
    <div class="space-y-2">
      <span class="text-xs uppercase tracking-wider font-semibold text-sky-400">Direct Donation</span>
      <h2 class="text-xl font-bold">Bantu Menjaga CATerm Tetap Mandiri, Cepat, dan Bebas Iklan</h2>
      <p class="text-neutral-300 text-sm leading-relaxed">
        CATerm dibangun dengan arsitektur native Rust dan Svelte modern tanpa analitik tersembunyi, tanpa server pihak ketiga yang mengintai, dan tanpa subscription paywall untuk fitur esensial. Setiap donasi langsung membantu pemeliharaan kode, cross-compilation package, dan riset fitur baru seperti AI Agent &amp; E2EE sync.
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
        class="px-4 py-3 bg-neutral-800 hover:bg-neutral-700 text-neutral-300 rounded-xl text-sm font-medium border border-neutral-700 transition-colors"
      >
        {copied ? '✓ Copied Link' : 'Copy PayPal URL'}
      </button>
    </div>
  </div>

  <!-- Dedication Box -->
  <div class="p-6 rounded-2xl bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 space-y-4">
    <div class="flex items-center gap-2 text-rose-500 font-semibold text-sm">
      <svg class="w-4 h-4 fill-current" viewBox="0 0 24 24">
        <path d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z"/>
      </svg>
      <span>Dedikasi &amp; Rasa Syukur</span>
    </div>
    <p class="text-sm text-neutral-600 dark:text-neutral-300 leading-relaxed">
      Karya ini didedikasikan dengan penuh cinta dan doa untuk keluarga tercinta:
    </p>
    <div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
      <div class="p-3 rounded-xl bg-neutral-50 dark:bg-neutral-950 border border-neutral-200 dark:border-neutral-800 text-center">
        <p class="text-xs text-neutral-400">Putra</p>
        <p class="font-bold text-neutral-900 dark:text-white text-sm mt-0.5">Umar Abdillah</p>
      </div>
      <div class="p-3 rounded-xl bg-neutral-50 dark:bg-neutral-950 border border-neutral-200 dark:border-neutral-800 text-center">
        <p class="text-xs text-neutral-400">Putra</p>
        <p class="font-bold text-neutral-900 dark:text-white text-sm mt-0.5">Fatih</p>
      </div>
      <div class="p-3 rounded-xl bg-neutral-50 dark:bg-neutral-950 border border-neutral-200 dark:border-neutral-800 text-center">
        <p class="text-xs text-neutral-400">Putra</p>
        <p class="font-bold text-neutral-900 dark:text-white text-sm mt-0.5">Harun</p>
      </div>
      <div class="p-3 rounded-xl bg-neutral-50 dark:bg-neutral-950 border border-neutral-200 dark:border-neutral-800 text-center">
        <p class="text-xs text-neutral-400">Putra</p>
        <p class="font-bold text-neutral-900 dark:text-white text-sm mt-0.5">Ibrahim</p>
      </div>
      <div class="p-3 rounded-xl bg-neutral-50 dark:bg-neutral-950 border border-neutral-200 dark:border-neutral-800 text-center">
        <p class="text-xs text-neutral-400">Putri</p>
        <p class="font-bold text-neutral-900 dark:text-white text-sm mt-0.5">Khadijah</p>
      </div>
      <div class="p-3 rounded-xl bg-neutral-50 dark:bg-neutral-950 border border-neutral-200 dark:border-neutral-800 text-center">
        <p class="text-xs text-neutral-400">Putri</p>
        <p class="font-bold text-neutral-900 dark:text-white text-sm mt-0.5">Maryam</p>
      </div>
      <div class="col-span-2 p-3 rounded-xl bg-rose-500/10 border border-rose-500/30 text-center">
        <p class="text-xs text-rose-500 dark:text-rose-400 font-medium">Istri Tercinta</p>
        <p class="font-bold text-neutral-900 dark:text-white text-sm mt-0.5">Rini</p>
      </div>
    </div>
  </div>

  <!-- Other Contribution Ways -->
  <div class="space-y-4">
    <h3 class="text-base font-semibold text-neutral-900 dark:text-white">Cara Kontribusi Lainnya</h3>
    <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
      <div class="p-5 rounded-xl bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 space-y-2">
        <div class="text-amber-500">
          <svg class="w-5 h-5 fill-current" viewBox="0 0 24 24"><path d="M12 17.27L18.18 21l-1.64-7.03L22 9.24l-7.19-.61L12 2 9.19 8.63 2 9.24l5.46 4.73L5.82 21z"/></svg>
        </div>
        <h4 class="font-bold text-neutral-900 dark:text-white text-sm">Star di GitHub</h4>
        <p class="text-xs text-neutral-500 dark:text-neutral-400 leading-relaxed">Berikan bintang pada repositori resmi untuk membantu lebih banyak engineer menemukan CATerm.</p>
      </div>

      <div class="p-5 rounded-xl bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 space-y-2">
        <div class="text-sky-500">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"/></svg>
        </div>
        <h4 class="font-bold text-neutral-900 dark:text-white text-sm">Laporkan Masalah</h4>
        <p class="text-xs text-neutral-500 dark:text-neutral-400 leading-relaxed">Menemukan bug atau memiliki ide fitur? Buat issue di GitHub repo untuk perbaikan berkelanjutan.</p>
      </div>

      <div class="p-5 rounded-xl bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 space-y-2">
        <div class="text-emerald-500">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8.684 13.342C8.886 12.938 9 12.482 9 12c0-.482-.114-.938-.316-1.342m0 2.684a3 3 0 110-2.684m0 2.684l6.632 3.316m-6.632-6l6.632-3.316m0 0a3 3 0 105.367-2.684 3 3 0 00-5.367 2.684zm0 9.316a3 3 0 105.368 2.684 3 3 0 00-5.368-2.684z"/></svg>
        </div>
        <h4 class="font-bold text-neutral-900 dark:text-white text-sm">Bagikan ke Komunitas</h4>
        <p class="text-xs text-neutral-500 dark:text-neutral-400 leading-relaxed">Rekomendasikan CATerm kepada rekan sysadmin, developer, dan cloud engineer di tim Anda.</p>
      </div>
    </div>
  </div>
</div>
