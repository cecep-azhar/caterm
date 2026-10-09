<script lang="ts">
  import type { Snippet } from 'svelte';

  export type BadgeVariant = 'neutral' | 'success' | 'warning' | 'danger' | 'brand';
  export type BadgeSize = 'xs' | 'sm';

  interface Props {
    variant?: BadgeVariant;
    size?: BadgeSize;
    class?: string;
    children?: Snippet;
  }

  let {
    variant = 'neutral',
    size = 'xs',
    class: customClass = '',
    children,
  }: Props = $props();

  const variantClasses: Record<BadgeVariant, string> = {
    neutral:
      'bg-neutral-100 dark:bg-neutral-800/80 text-neutral-700 dark:text-neutral-300 border border-neutral-300 dark:border-neutral-700',
    success:
      'bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border border-emerald-500/30',
    warning:
      'bg-amber-500/10 text-amber-600 dark:text-amber-400 border border-amber-500/30',
    danger:
      'bg-rose-500/10 text-rose-600 dark:text-rose-400 border border-rose-500/30',
    brand:
      'bg-[var(--ca-brand)]/15 text-[var(--ca-brand)] border border-[var(--ca-brand)]/35',
  };

  const sizeClasses: Record<BadgeSize, string> = {
    xs: 'text-[10px] px-1.5 py-0.5 rounded-md font-mono font-medium gap-1',
    sm: 'text-xs px-2 py-0.5 rounded-lg font-medium gap-1.5',
  };
</script>

<span
  class="inline-flex items-center justify-center select-none tracking-tight {variantClasses[variant]} {sizeClasses[size]} {customClass}"
>
  {@render children?.()}
</span>
