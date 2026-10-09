<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    open?: boolean;
    title?: string;
    description?: string;
    size?: 'sm' | 'md' | 'lg' | 'xl' | 'full';
    closeOnEsc?: boolean;
    closeOnBackdrop?: boolean;
    showCloseButton?: boolean;
    class?: string;
    onclose?: () => void;
    children?: Snippet;
    header?: Snippet;
    footer?: Snippet;
  }

  let {
    open = $bindable(false),
    title = '',
    description = '',
    size = 'md',
    closeOnEsc = true,
    closeOnBackdrop = true,
    showCloseButton = true,
    class: customClass = '',
    onclose,
    children,
    header,
    footer,
  }: Props = $props();

  const sizeClasses: Record<string, string> = {
    sm: 'max-w-sm',
    md: 'max-w-md',
    lg: 'max-w-xl',
    xl: 'max-w-3xl',
    full: 'max-w-5xl w-[95vw]',
  };

  function handleClose() {
    open = false;
    onclose?.();
  }

  function handleBackdropClick(e: MouseEvent) {
    if (closeOnBackdrop && e.target === e.currentTarget) {
      handleClose();
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (closeOnEsc && e.key === 'Escape') {
      handleClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeyDown} />

{#if open}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70 backdrop-blur-xs transition-opacity duration-200"
    onclick={handleBackdropClick}
    role="dialog" tabindex="-1"
    aria-modal="true"
  >
    <div
      class="w-full bg-[#121217] border border-[#272732] rounded-2xl shadow-2xl overflow-hidden flex flex-col max-h-[90vh] text-[#EDEDED] font-sans animate-in fade-in zoom-in-95 duration-150 {sizeClasses[size]} {customClass}"
      onclick={(e) => e.stopPropagation()}
      role="document"
    >
      <!-- Modal Header -->
      {#if header}
        {@render header()}
      {:else if title || showCloseButton}
        <div class="px-6 py-4 border-b border-[#272732] flex items-center justify-between gap-3 bg-[#18181F]/50">
          <div class="min-w-0 flex-1">
            {#if title}
              <h3 class="text-base font-semibold text-white truncate">{title}</h3>
            {/if}
            {#if description}
              <p class="text-xs text-neutral-400 mt-0.5">{description}</p>
            {/if}
          </div>
          {#if showCloseButton}
            <button
              type="button"
              onclick={handleClose}
              class="text-neutral-400 hover:text-white p-1 rounded-lg hover:bg-neutral-800 transition cursor-pointer"
              aria-label="Close dialog"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          {/if}
        </div>
      {/if}

      <!-- Modal Body -->
      <div class="px-6 py-5 overflow-y-auto flex-1">
        {@render children?.()}
      </div>

      <!-- Modal Footer -->
      {#if footer}
        <div class="px-6 py-3.5 border-t border-[#272732] bg-[#18181F]/40 flex items-center justify-end gap-2.5">
          {@render footer()}
        </div>
      {/if}
    </div>
  </div>
{/if}
