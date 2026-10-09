<script lang="ts">
  let { active }: { active: 'routing' | 'personas' | 'habits' | 'skills' | 'quick' } = $props();
</script>

<div class="border-b border-neutral-200 dark:border-neutral-800 flex items-center gap-6 overflow-x-auto text-xs font-mono font-medium select-none" role="tablist">
  <a
    href="/settings/ai-routing"
    role="tab"
    aria-selected={active === 'routing'}
    class="pb-3 flex items-center gap-2 whitespace-nowrap transition-colors border-b-2 {active === 'routing'
      ? 'border-[var(--ca-brand,#ef4444)] text-white font-semibold'
      : 'border-transparent text-neutral-400 hover:text-white'}"
  >
    <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" /></svg>
    <span>Providers & Routing</span>
  </a>
  <a
    href="/settings/ai-skills"
    role="tab"
    aria-selected={active === 'skills'}
    class="pb-3 flex items-center gap-2 whitespace-nowrap transition-colors border-b-2 {active === 'skills'
      ? 'border-[var(--ca-brand,#ef4444)] text-white font-semibold'
      : 'border-transparent text-neutral-400 hover:text-white'}"
  >
    <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19.428 15.428a2 2 0 00-1.022-.547l-2.387-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z" /></svg>
    <span>Custom Skills (@skills)</span>
  </a>
  <a
    href="/settings/ai-personas"
    role="tab"
    aria-selected={active === 'personas'}
    class="pb-3 flex items-center gap-2 whitespace-nowrap transition-colors border-b-2 {active === 'personas'
      ? 'border-[var(--ca-brand,#ef4444)] text-white font-semibold'
      : 'border-transparent text-neutral-400 hover:text-white'}"
  >
    <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" /></svg>
    <span>Personas & Rules</span>
  </a>
  <a
    href="/settings/ai-habits"
    role="tab"
    aria-selected={active === 'habits'}
    class="pb-3 flex items-center gap-2 whitespace-nowrap transition-colors border-b-2 {active === 'habits'
      ? 'border-[var(--ca-brand,#ef4444)] text-white font-semibold'
      : 'border-transparent text-neutral-400 hover:text-white'}"
  >
    <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" /></svg>
    <span>Habit Memory (FTS5)</span>
  </a>
</div>
