<script lang="ts">
  import { getLocale, setLocale, type Locale } from '$lib/i18n/index.svelte';

  interface Props {
    compact?: boolean;
    class?: string;
  }

  let {
    compact = false,
    class: customClass = '',
  }: Props = $props();

  const currentLocale = $derived(getLocale());
</script>

<div class="inline-flex items-center rounded-lg border border-neutral-200 dark:border-[#272732] bg-neutral-100 dark:bg-[#121217] p-0.5 text-xs font-medium font-sans select-none {customClass}">
  <button
    type="button"
    onclick={() => setLocale('en')}
    class="px-2 py-1 rounded-md transition-all cursor-pointer {currentLocale === 'en' ? 'bg-white dark:bg-[#1E1E26] text-neutral-900 dark:text-white shadow-xs font-semibold' : 'text-neutral-500 hover:text-neutral-800 dark:hover:text-neutral-300'}"
    title="English"
  >
    {compact ? 'EN' : 'English'}
  </button>
  <button
    type="button"
    onclick={() => setLocale('id')}
    class="px-2 py-1 rounded-md transition-all cursor-pointer {currentLocale === 'id' ? 'bg-white dark:bg-[#1E1E26] text-neutral-900 dark:text-white shadow-xs font-semibold' : 'text-neutral-500 hover:text-neutral-800 dark:hover:text-neutral-300'}"
    title="Bahasa Indonesia"
  >
    {compact ? 'ID' : 'Indonesia'}
  </button>
</div>
