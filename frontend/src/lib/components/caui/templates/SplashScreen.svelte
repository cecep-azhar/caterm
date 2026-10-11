<script lang="ts">
  import { onMount } from 'svelte';
  import { Icon } from '@cecepazhar/caui';

  interface Props {
    appName?: string;
    version?: string;
    showProgress?: boolean;
    duration?: number; // duration in ms to auto complete
    oncomplete?: () => void;
    class?: string;
  }

  let {
    appName = 'CATerm',
    version = '2.1.20',
    showProgress = true,
    duration = 2400,
    oncomplete,
    class: customClass = '',
  }: Props = $props();

  const quotes = [
    { text: 'Talk is cheap. Show me the code.', author: 'Linus Torvalds' },
    { text: 'Simplicity is prerequisite for reliability.', author: 'Edsger W. Dijkstra' },
    { text: 'The only way to do great work is to love what you do.', author: 'Steve Jobs' },
    { text: "It's easier to ask forgiveness than it is to get permission.", author: 'Grace Hopper' },
    { text: 'Premature optimization is the root of all evil.', author: 'Donald Knuth' },
  ];

  let currentQuote = $state(quotes[0]);
  let progress = $state(0);

  onMount(() => {
    // Pick random quote
    currentQuote = quotes[Math.floor(Math.random() * quotes.length)];

    if (showProgress) {
      const step = 20; // 20ms update interval
      const increment = 100 / (duration / step);
      const timer = setInterval(() => {
        progress += increment;
        if (progress >= 100) {
          progress = 100;
          clearInterval(timer);
          setTimeout(() => oncomplete?.(), 250);
        }
      }, step);

      return () => clearInterval(timer);
    }
  });
</script>

<div class="relative w-full h-full min-h-[460px] bg-[#0A0A0C] text-[#EDEDED] flex flex-col justify-between p-8 sm:p-12 overflow-hidden rounded-2xl border border-[#272732] font-sans select-none {customClass}">
  <!-- Ambient Background Glow -->
  <div class="absolute -top-32 -left-32 w-80 h-80 bg-[var(--ca-brand)]/15 rounded-full blur-3xl pointer-events-none"></div>
  <div class="absolute -bottom-32 -right-32 w-80 h-80 bg-blue-600/10 rounded-full blur-3xl pointer-events-none"></div>

  <!-- Top Header with Status -->
  <div class="relative z-10 flex items-center justify-between">
    <div class="flex items-center gap-2 text-xs font-mono text-neutral-400">
      <span class="w-2 h-2 rounded-full bg-emerald-500 animate-pulse"></span>
      <span>SYSTEM BOOTSTRAP</span>
    </div>
    <div class="text-xs font-mono px-2 py-0.5 rounded bg-neutral-900 border border-neutral-800 text-neutral-400">
      v{version}
    </div>
  </div>

  <!-- Center Hero: Dual Wing CA Icon & Typography -->
  <div class="relative z-10 flex flex-col items-center justify-center my-auto space-y-6 text-center">
    <!-- Stylized Dual Wing Logo -->
    <div class="relative group">
      <div class="w-20 h-20 rounded-2xl bg-[#121217] border border-[#272732] shadow-2xl flex items-center justify-center relative overflow-hidden">
        <div class="absolute inset-0 bg-gradient-to-tr from-[var(--ca-brand)]/20 via-transparent to-transparent"></div>
        <svg class="w-10 h-10 text-[var(--ca-brand)]" viewBox="0 0 32 32" fill="none" stroke="currentColor">
          <path d="M6 10L16 4L26 10L26 22L16 28L6 22Z" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
          <path d="M11 13L16 10L21 13M16 10V22" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
      </div>
    </div>

    <div>
      <h1 class="text-3xl font-extrabold tracking-tight text-white flex items-center justify-center gap-2">
        {appName}
        <span class="text-xs px-2 py-0.5 rounded-full font-bold bg-[var(--ca-brand)]/20 text-[var(--ca-brand)] border border-[var(--ca-brand)]/40 font-mono">PRO</span>
      </h1>
      <p class="text-xs text-neutral-400 mt-1.5 max-w-sm">
        Next-generation AI terminal emulator and secure remote cloud infrastructure manager.
      </p>
    </div>

    <!-- Quote Block -->
    <div class="max-w-md p-4 rounded-xl bg-[#121217]/80 border border-[#272732] text-xs text-neutral-300">
      <p class="italic font-light leading-relaxed">"{currentQuote.text}"</p>
      <p class="mt-2 text-[11px] font-mono font-medium text-[var(--ca-brand)]">— {currentQuote.author}</p>
    </div>
  </div>

  <!-- Bottom Progress & Telemetry -->
  <div class="relative z-10 space-y-3 max-w-md mx-auto w-full">
    {#if showProgress}
      <div class="space-y-1.5">
        <div class="flex items-center justify-between text-[11px] font-mono text-neutral-400">
          <span>INITIALIZING ENCRYPTED VAULT</span>
          <span>{Math.round(progress)}%</span>
        </div>
        <div class="w-full h-1.5 bg-neutral-900 rounded-full overflow-hidden border border-neutral-800">
          <div
            class="h-full bg-[var(--ca-brand)] transition-all duration-75 ease-out rounded-full shadow-[0_0_8px_var(--ca-brand)]"
            style="width: {progress}%;"
          ></div>
        </div>
      </div>
    {/if}

    <div class="flex items-center justify-between text-[10px] text-neutral-400 pt-2 border-t border-neutral-800/80">
      <span>Zero-Knowledge Architecture</span>
      <span>CA Design System 1.0</span>
    </div>
  </div>
</div>
