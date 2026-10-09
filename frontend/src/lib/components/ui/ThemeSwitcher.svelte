<script lang="ts">
  import Icon from './Icon.svelte';

  export interface BrandAccent {
    name: string;
    hex: string;
  }

  interface Props {
    showAccentPicker?: boolean;
    class?: string;
  }

  let {
    showAccentPicker = true,
    class: customClass = '',
  }: Props = $props();

  const brandAccents: BrandAccent[] = [
    { name: 'Monochrome (Zinc)', hex: '#71717a' },
    { name: 'Cyan (CATerm / CAMark)', hex: '#06b6d4' },
    { name: 'Emerald (CACash)', hex: '#10b981' },
    { name: 'Violet (CAStudio)', hex: '#8b5cf6' },
    { name: 'Rose (Alert)', hex: '#f43f5e' },
    { name: 'Blue (Core)', hex: '#3b82f6' },
  ];

  let isDark = $state(true);
  let selectedAccent = $state('#06b6d4');

  function toggleTheme(dark: boolean) {
    isDark = dark;
    if (typeof document !== 'undefined') {
      if (dark) {
        document.documentElement.classList.add('dark');
        localStorage.setItem('theme', 'dark');
      } else {
        document.documentElement.classList.remove('dark');
        localStorage.setItem('theme', 'light');
      }
    }
  }

  function setBrandAccent(hex: string) {
    selectedAccent = hex;
    if (typeof document !== 'undefined') {
      document.documentElement.style.setProperty('--ca-brand', hex);
      localStorage.setItem('ca-brand', hex);
    }
  }
</script>

<div class="flex items-center gap-3 font-sans text-xs select-none {customClass}">
  <!-- Dark / Light Toggle -->
  <div class="inline-flex items-center p-0.5 rounded-lg border border-neutral-200 dark:border-[#272732] bg-neutral-100 dark:bg-[#121217]">
    <button
      type="button"
      onclick={() => toggleTheme(false)}
      class="p-1.5 rounded-md transition-all cursor-pointer {!isDark ? 'bg-white text-amber-500 shadow-xs' : 'text-neutral-500 hover:text-neutral-900 dark:hover:text-white'}"
      title="Light Mode"
      aria-label="Light Mode"
    >
      <Icon name="sun" size={14} />
    </button>
    <button
      type="button"
      onclick={() => toggleTheme(true)}
      class="p-1.5 rounded-md transition-all cursor-pointer {isDark ? 'bg-neutral-800 text-white shadow-xs' : 'text-neutral-500 hover:text-neutral-900 dark:hover:text-white'}"
      title="Dark Mode"
      aria-label="Dark Mode"
    >
      <Icon name="moon" size={14} />
    </button>
  </div>

  <!-- Pro Brand Accent Swatches -->
  {#if showAccentPicker}
    <div class="flex items-center gap-1.5 pl-2 border-l border-neutral-200 dark:border-[#272732]">
      {#each brandAccents as b}
        <button
          type="button"
          onclick={() => setBrandAccent(b.hex)}
          title={b.name}
          class="w-4 h-4 rounded-full border transition-all cursor-pointer relative {selectedAccent === b.hex ? 'ring-2 ring-white/60 scale-110 border-white' : 'border-black/40 opacity-70 hover:opacity-100'}"
          style="background-color: {b.hex};"
        >
          {#if selectedAccent === b.hex}
            <span class="absolute inset-0 flex items-center justify-center text-[8px] text-white font-bold leading-none">✓</span>
          {/if}
        </button>
      {/each}
    </div>
  {/if}
</div>
