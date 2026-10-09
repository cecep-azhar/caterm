<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    collapsed?: boolean;
    class?: string;
    header?: Snippet;
    footer?: Snippet;
    children?: Snippet;
  }

  let {
    collapsed = false,
    class: customClass = '',
    header,
    footer,
    children,
  }: Props = $props();
</script>

<aside
  class="flex flex-col h-full bg-white dark:bg-[#0E0E12] border-r border-neutral-200 dark:border-[#272732] select-none transition-all duration-200 {collapsed ? 'w-16' : 'w-64'} {customClass}"
  aria-label="Sidebar navigation"
>
  {#if header}
    <div class="p-3 border-b border-neutral-200 dark:border-[#272732] flex items-center {collapsed ? 'justify-center' : 'justify-between'}">
      {@render header()}
    </div>
  {/if}

  <nav class="flex-1 overflow-y-auto p-2 space-y-1">
    {@render children?.()}
  </nav>

  {#if footer}
    <div class="p-2 border-t border-neutral-200 dark:border-[#272732]">
      {@render footer()}
    </div>
  {/if}
</aside>
