<script lang="ts">
  import Logo from './Logo.svelte';
  import { APP_VERSION, REPO_URL } from '$lib/appInfo';

  let { onClose }: { onClose: () => void } = $props();
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && onClose()} />

<div class="fixed inset-0 z-[100] flex items-center justify-center p-4">
  <button
    type="button"
    class="absolute inset-0 bg-black/60 backdrop-blur-sm cursor-default"
    aria-label="Close About dialog"
    onclick={onClose}
  ></button>

  <div
    role="dialog"
    aria-modal="true"
    aria-labelledby="about-title"
    class="no-drag relative w-full max-w-sm rounded-2xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-[#141414] shadow-2xl p-6 text-center"
  >
    <button
      type="button"
      onclick={onClose}
      class="absolute top-3 right-3 p-1.5 rounded-lg text-neutral-400 hover:text-neutral-900 dark:hover:text-white hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors"
      aria-label="Close"
    >
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>
    </button>

    <div class="flex justify-center mb-3">
      <Logo size={48} mode="brand" />
    </div>
    <h2 id="about-title" class="text-lg font-bold text-neutral-900 dark:text-white">CATerm</h2>
    <p class="mt-1 text-xs font-mono text-sky-600 dark:text-sky-400">v{APP_VERSION}</p>
    <p class="mt-3 text-sm text-neutral-500 dark:text-neutral-400">
      Local-first SSH manager with an encrypted vault, SFTP, and an AI ops assistant.
    </p>

    <dl class="mt-5 text-xs text-left divide-y divide-neutral-100 dark:divide-neutral-800 border-y border-neutral-100 dark:border-neutral-800">
      <div class="flex justify-between py-2">
        <dt class="text-neutral-500">License</dt>
        <dd class="text-neutral-900 dark:text-neutral-200 font-medium">MIT</dd>
      </div>
      <div class="flex justify-between py-2">
        <dt class="text-neutral-500">Source</dt>
        <dd>
          <a href={REPO_URL} class="text-sky-600 dark:text-sky-400 hover:underline font-mono">cecep-azhar/caterm</a>
        </dd>
      </div>
      <div class="flex justify-between py-2">
        <dt class="text-neutral-500">Built by</dt>
        <dd class="text-neutral-900 dark:text-neutral-200 font-medium">Fathforce</dd>
      </div>
    </dl>

    <p class="mt-4 text-[11px] text-neutral-400 dark:text-neutral-500">
      © {new Date().getFullYear()} Cecep Saeful Azhar Hidayat · Fathforce
    </p>
  </div>
</div>
