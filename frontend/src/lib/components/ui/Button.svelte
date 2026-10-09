<script lang="ts">
  import type { Snippet } from 'svelte';

  export type ButtonVariant = 'primary' | 'secondary' | 'outline' | 'ghost' | 'danger' | 'brand';
  export type ButtonSize = 'sm' | 'md' | 'lg' | 'icon';

  interface Props {
    variant?: ButtonVariant;
    size?: ButtonSize;
    disabled?: boolean;
    loading?: boolean;
    type?: 'button' | 'submit' | 'reset';
    title?: string;
    class?: string;
    onclick?: (e: MouseEvent) => void;
    children?: Snippet;
  }

  let {
    variant = 'secondary',
    size = 'md',
    disabled = false,
    loading = false,
    type = 'button',
    title,
    class: customClass = '',
    onclick,
    children,
  }: Props = $props();

  const variantClasses: Record<ButtonVariant, string> = {
    primary:
      'bg-white text-neutral-950 font-semibold hover:bg-neutral-200 border border-white dark:bg-white dark:text-neutral-950 dark:hover:bg-neutral-200 shadow-sm',
    secondary:
      'bg-neutral-100 dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 text-neutral-800 dark:text-neutral-200 hover:bg-neutral-200 dark:hover:bg-neutral-800/80',
    outline:
      'bg-transparent border border-neutral-300 dark:border-neutral-700/80 text-neutral-700 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800/60 hover:border-neutral-400 dark:hover:border-neutral-500',
    ghost:
      'bg-transparent text-neutral-600 dark:text-neutral-400 hover:bg-neutral-100 dark:hover:bg-neutral-800/60 hover:text-neutral-900 dark:hover:text-neutral-100',
    danger:
      'bg-rose-500/10 border border-rose-500/30 text-rose-600 dark:text-rose-400 hover:bg-rose-500/20 hover:border-rose-500/50',
    brand:
      'bg-[var(--ca-brand)]/10 border border-[var(--ca-brand)]/40 text-[var(--ca-brand)] hover:bg-[var(--ca-brand)]/20 hover:border-[var(--ca-brand)]/60 shadow-xs',
  };

  const sizeClasses: Record<ButtonSize, string> = {
    sm: 'px-2.5 py-1 text-xs rounded-lg gap-1.5',
    md: 'px-3.5 py-1.5 text-xs font-semibold rounded-lg gap-2',
    lg: 'px-4 py-2 text-sm font-bold rounded-xl gap-2.5',
    icon: 'p-1.5 rounded-lg justify-center',
  };
</script>

<button
  {type}
  {title}
  disabled={disabled || loading}
  {onclick}
  class="inline-flex items-center justify-center font-sans select-none transition-all duration-150 cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed disabled:pointer-events-none active:scale-[0.98] {variantClasses[variant]} {sizeClasses[size]} {customClass}"
>
  {#if loading}
    <svg class="animate-spin -ml-0.5 mr-1.5 h-3.5 w-3.5 shrink-0" fill="none" viewBox="0 0 24 24">
      <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
      <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
    </svg>
  {/if}
  {@render children?.()}
</button>
