<script lang="ts">
  import type { BlastShieldRisk } from '$lib/api/proSecurity';

  let {
    isOpen = $bindable(false),
    command = '',
    risk = null,
    onConfirm,
    onCancel
  }: {
    isOpen: boolean;
    command: string;
    risk: BlastShieldRisk | null;
    onConfirm: () => void;
    onCancel: () => void;
  } = $props();

  let isBlocked = $derived(risk?.level === 'blocked');
</script>

{#if isOpen && risk}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-sm p-4 animate-in fade-in duration-200">
    <div class="w-full max-w-lg rounded-xl border border-red-500/30 bg-[#121217] shadow-2xl p-6 text-white space-y-4">
      <div class="flex items-center gap-3">
        <div class="p-2.5 rounded-lg bg-red-500/10 border border-red-500/20 text-red-400">
          <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
          </svg>
        </div>
        <div>
          <h3 class="text-base font-bold text-red-400 flex items-center gap-2">
            CATerm Blast Shield Intercept
            <span class="text-xs px-2 py-0.5 rounded bg-red-500/20 text-red-300 font-mono uppercase tracking-wide">
              {risk.level}
            </span>
          </h3>
          <p class="text-xs text-neutral-400">Potentially destructive command detected before PTY dispatch</p>
        </div>
      </div>

      <div class="p-3 rounded-lg bg-black/40 border border-neutral-800 font-mono text-xs text-neutral-200 break-all overflow-x-auto">
        <code>{command}</code>
      </div>

      <div class="space-y-2 text-xs">
        <div class="p-3 rounded-lg bg-red-950/30 border border-red-800/30 text-red-200/90 leading-relaxed">
          <span class="font-semibold text-red-300">Risk Analysis:</span> {risk.reason}
        </div>

        {#if risk.suggestedAlternative}
          <div class="p-3 rounded-lg bg-neutral-900 border border-neutral-800 text-neutral-300 leading-relaxed">
            <span class="font-semibold text-neutral-200">Recommendation:</span> {risk.suggestedAlternative}
          </div>
        {/if}
      </div>

      <div class="pt-2 flex items-center justify-end gap-3">
        <button
          type="button"
          class="px-4 py-2 text-xs font-medium text-neutral-300 hover:text-white bg-neutral-800 hover:bg-neutral-700 rounded-lg transition-colors"
          onclick={onCancel}
        >
          Abort Execution
        </button>

        {#if !isBlocked}
          <button
            type="button"
            class="px-4 py-2 text-xs font-semibold text-white bg-red-600 hover:bg-red-500 active:scale-95 rounded-lg transition-all shadow-lg shadow-red-600/20"
            onclick={onConfirm}
          >
            I Understand, Execute Anyway
          </button>
        {:else}
          <div class="text-[11px] text-red-400/90 font-mono italic">
            Execution strictly blocked for system safety.
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}
