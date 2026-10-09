<script lang="ts">
  import type { PasteCheckResult } from '$lib/api/proSecurity';

  let {
    isOpen = $bindable(false),
    pasteResult = null,
    onPasteSanitized,
    onPasteOriginal,
    onCancel
  }: {
    isOpen: boolean;
    pasteResult: PasteCheckResult | null;
    onPasteSanitized: (text: string) => void;
    onPasteOriginal: (text: string) => void;
    onCancel: () => void;
  } = $props();
</script>

{#if isOpen && pasteResult && pasteResult.containsSecret}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-sm p-4 animate-in fade-in duration-200">
    <div class="w-full max-w-lg rounded-xl border border-amber-500/30 bg-[#121217] shadow-2xl p-6 text-white space-y-4">
      <div class="flex items-center gap-3">
        <div class="p-2.5 rounded-lg bg-amber-500/10 border border-amber-500/20 text-amber-400">
          <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
          </svg>
        </div>
        <div>
          <h3 class="text-base font-bold text-amber-400 flex items-center gap-2">
            CATerm Paste Sentinel Alert
            <span class="text-xs px-2 py-0.5 rounded bg-amber-500/20 text-amber-300 font-mono uppercase tracking-wide">
              {pasteResult.secrets.length} Secret{pasteResult.secrets.length > 1 ? 's' : ''} Found
            </span>
          </h3>
          <p class="text-xs text-neutral-400">Sensitive credentials detected in clipboard payload</p>
        </div>
      </div>

      <div class="space-y-2 max-h-48 overflow-y-auto pr-1">
        {#each pasteResult.secrets as secret}
          <div class="p-3 rounded-lg bg-amber-950/20 border border-amber-800/30 text-xs space-y-1">
            <div class="flex items-center justify-between font-semibold text-amber-300">
              <span>{secret.title}</span>
              <span class="font-mono text-[11px] text-amber-400/80">{secret.matchedSnippetMasked}</span>
            </div>
            <p class="text-neutral-400 text-[11px] leading-relaxed">{secret.description}</p>
          </div>
        {/each}
      </div>

      <div class="p-3 rounded-lg bg-neutral-900 border border-neutral-800 text-xs space-y-1.5 text-neutral-300">
        <div class="font-semibold text-neutral-200">Recommended Safe Action:</div>
        <p class="text-[11px] text-neutral-400 leading-relaxed">
          Prefix command with a leading whitespace (<code class="text-amber-300">HISTCONTROL=ignorespace</code>) to avoid writing secrets to shell history logs (<code class="text-neutral-400">~/.bash_history</code> / <code class="text-neutral-400">~/.zsh_history</code>).
        </p>
      </div>

      <div class="pt-2 flex items-center justify-end gap-2 flex-wrap">
        <button
          type="button"
          class="px-3.5 py-2 text-xs font-medium text-neutral-400 hover:text-white bg-neutral-800 hover:bg-neutral-700 rounded-lg transition-colors"
          onclick={onCancel}
        >
          Cancel
        </button>

        <button
          type="button"
          class="px-3.5 py-2 text-xs font-medium text-amber-300 hover:text-white bg-amber-950/40 hover:bg-amber-900/50 border border-amber-800/50 rounded-lg transition-colors"
          onclick={() => onPasteOriginal(pasteResult?.sanitizedText.trim() ?? '')}
        >
          Paste Raw (History Visible)
        </button>

        <button
          type="button"
          class="px-4 py-2 text-xs font-semibold text-black bg-amber-400 hover:bg-amber-300 active:scale-95 rounded-lg transition-all shadow-lg shadow-amber-400/20"
          onclick={() => onPasteSanitized(pasteResult?.sanitizedText ?? '')}
        >
          Paste with Space Prefix (Safe)
        </button>
      </div>
    </div>
  </div>
{/if}
