<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    title?: string;
    description?: string;
    class?: string;
    headerAction?: Snippet;
    children?: Snippet;
  }

  let {
    title,
    description,
    class: customClass = '',
    headerAction,
    children,
  }: Props = $props();
</script>

<div
  class="bg-white dark:bg-[#121217] border border-neutral-200 dark:border-[#272732] rounded-xl p-5 shadow-xs transition-colors {customClass}"
>
  {#if title || headerAction}
    <div class="flex items-start justify-between gap-4 mb-4 pb-3 border-b border-neutral-100 dark:border-[#272732]/80">
      <div>
        {#if title}
          <h3 class="font-bold text-neutral-900 dark:text-white text-xs sm:text-sm tracking-tight">{title}</h3>
        {/if}
        {#if description}
          <p class="text-[11px] text-neutral-500 dark:text-neutral-400 mt-0.5 leading-relaxed">{description}</p>
        {/if}
      </div>
      {#if headerAction}
        <div class="shrink-0 flex items-center gap-2">
          {@render headerAction()}
        </div>
      {/if}
    </div>
  {/if}

  {@render children?.()}
</div>
