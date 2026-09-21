<script lang="ts">
  import type { AutocompleteItem } from '$lib/data/terminalCommands';

  let {
    suggestions = [],
    selectedIndex = 0,
    onSelect,
    onClose
  }: {
    suggestions: AutocompleteItem[];
    selectedIndex: number;
    onSelect: (item: AutocompleteItem) => void;
    onClose: () => void;
  } = $props();
</script>

{#if suggestions.length > 0}
  <div
    class="absolute left-4 bottom-4 z-40 max-w-lg min-w-[320px] bg-neutral-900/95 dark:bg-[#121215]/95 backdrop-blur-md border border-neutral-700/70 dark:border-neutral-800 rounded-lg shadow-2xl overflow-hidden font-mono text-xs select-none animate-in fade-in zoom-in-95 duration-100"
  >
    <!-- Header bar -->
    <div class="px-2.5 py-1.5 bg-neutral-800/80 dark:bg-neutral-900/80 border-b border-neutral-700/50 flex items-center justify-between text-[10px] text-neutral-400">
      <div class="flex items-center gap-1.5 font-semibold text-neutral-300">
        <span class="w-1.5 h-1.5 rounded-full bg-sky-400 animate-pulse"></span>
        <span>Smart Autocomplete</span>
      </div>
      <div class="flex items-center gap-2">
        <span class="text-[9px] px-1.5 py-0.5 rounded bg-neutral-700/50 text-neutral-300">Tab / Enter to complete</span>
        <button
          onclick={onClose}
          class="hover:text-neutral-200 transition-colors p-0.5 rounded"
          title="Dismiss"
          aria-label="Dismiss"
        >
          ✕
        </button>
      </div>
    </div>

    <!-- Suggestion items list -->
    <div class="max-h-56 overflow-y-auto divide-y divide-neutral-800/40 py-1">
      {#each suggestions as item, idx}
        <button
          type="button"
          class="w-full px-3 py-1.5 flex items-center justify-between gap-3 text-left transition-colors {idx === selectedIndex ? 'bg-sky-600/30 text-white border-l-2 border-sky-400' : 'text-neutral-300 hover:bg-neutral-800/50'}"
          onclick={() => onSelect(item)}
        >
          <div class="flex items-center gap-2 min-w-0 flex-1">
            <!-- Badge type -->
            {#if item.type === 'history'}
              <span class="px-1.5 py-0.5 rounded text-[9px] font-semibold bg-amber-500/20 text-amber-300 border border-amber-500/30 shrink-0">
                HIST
              </span>
            {:else if item.type === 'snippet'}
              <span class="px-1.5 py-0.5 rounded text-[9px] font-semibold bg-emerald-500/20 text-emerald-300 border border-emerald-500/30 shrink-0">
                SNIP
              </span>
            {:else}
              <span class="px-1.5 py-0.5 rounded text-[9px] font-semibold bg-sky-500/20 text-sky-300 border border-sky-500/30 shrink-0">
                CMD
              </span>
            {/if}

            <span class="font-mono text-xs font-semibold truncate text-white">
              {item.text}
            </span>
          </div>

          {#if item.desc || item.sourceLabel}
            <span class="text-[11px] text-neutral-400 dark:text-neutral-400 truncate max-w-[200px] text-right">
              {item.sourceLabel ? `[${item.sourceLabel}] ` : ''}{item.desc || ''}
            </span>
          {/if}
        </button>
      {/each}
    </div>
  </div>
{/if}
