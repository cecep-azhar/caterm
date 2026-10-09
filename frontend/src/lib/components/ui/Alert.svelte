<script lang="ts">
  import type { Snippet } from 'svelte';

  export type AlertVariant = 'info' | 'success' | 'warning' | 'error';

  interface Props {
    variant?: AlertVariant;
    title?: string;
    description?: string;
    dismissible?: boolean;
    class?: string;
    ondismiss?: () => void;
    children?: Snippet;
    action?: Snippet;
  }

  let {
    variant = 'info',
    title = '',
    description = '',
    dismissible = false,
    class: customClass = '',
    ondismiss,
    children,
    action,
  }: Props = $props();

  let visible = $state(true);

  function handleDismiss() {
    visible = false;
    ondismiss?.();
  }

  const variantStyles: Record<AlertVariant, { bg: string; border: string; text: string; iconColor: string; path: string }> = {
    info: {
      bg: 'bg-sky-500/10 dark:bg-sky-500/10',
      border: 'border-sky-500/30',
      text: 'text-sky-800 dark:text-sky-300',
      iconColor: 'text-sky-500',
      path: 'M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z',
    },
    success: {
      bg: 'bg-emerald-500/10 dark:bg-emerald-500/10',
      border: 'border-emerald-500/30',
      text: 'text-emerald-800 dark:text-emerald-300',
      iconColor: 'text-emerald-500',
      path: 'M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z',
    },
    warning: {
      bg: 'bg-amber-500/10 dark:bg-amber-500/10',
      border: 'border-amber-500/30',
      text: 'text-amber-800 dark:text-amber-300',
      iconColor: 'text-amber-500',
      path: 'M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z',
    },
    error: {
      bg: 'bg-rose-500/10 dark:bg-rose-500/10',
      border: 'border-rose-500/30',
      text: 'text-rose-800 dark:text-rose-300',
      iconColor: 'text-rose-500',
      path: 'M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z',
    },
  };

  const current = $derived(variantStyles[variant]);
</script>

{#if visible}
  <div
    class="flex items-start gap-3 p-3.5 rounded-xl border font-sans text-xs transition-all {current.bg} {current.border} {customClass}"
    role="alert"
  >
    <div class="shrink-0 mt-0.5 {current.iconColor}">
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={current.path} />
      </svg>
    </div>

    <div class="flex-1 min-w-0">
      {#if title}
        <h4 class="font-semibold mb-0.5 {current.text}">{title}</h4>
      {/if}
      {#if description}
        <p class="opacity-90 leading-relaxed {current.text}">{description}</p>
      {/if}
      {@render children?.()}
    </div>

    {#if action}
      <div class="shrink-0 ml-2">
        {@render action()}
      </div>
    {/if}

    {#if dismissible}
      <button
        type="button"
        onclick={handleDismiss}
        class="shrink-0 p-1 rounded-md opacity-60 hover:opacity-100 transition cursor-pointer text-current"
        aria-label="Dismiss alert"
      >
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    {/if}
  </div>
{/if}
