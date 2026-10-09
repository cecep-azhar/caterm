<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    value?: string;
    type?: string;
    placeholder?: string;
    disabled?: boolean;
    readonly?: boolean;
    id?: string;
    class?: string;
    leadingIcon?: Snippet;
    trailingAction?: Snippet;
    oninput?: (e: Event) => void;
    onkeydown?: (e: KeyboardEvent) => void;
  }

  let {
    value = $bindable(''),
    type = 'text',
    placeholder = '',
    disabled = false,
    readonly = false,
    id,
    class: customClass = '',
    leadingIcon,
    trailingAction,
    oninput,
    onkeydown,
  }: Props = $props();
</script>

<div class="relative flex items-center w-full {customClass}">
  {#if leadingIcon}
    <div class="absolute left-3 flex items-center pointer-events-none text-neutral-400">
      {@render leadingIcon()}
    </div>
  {/if}

  <input
    {id}
    {type}
    {placeholder}
    {disabled}
    {readonly}
    bind:value
    {oninput}
    {onkeydown}
    class="w-full bg-white dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 focus:border-neutral-500 dark:focus:border-neutral-600 text-neutral-900 dark:text-white placeholder-neutral-400 dark:placeholder-neutral-500 rounded-lg text-xs py-2 {leadingIcon ? 'pl-9' : 'pl-3'} {trailingAction ? 'pr-9' : 'pr-3'} outline-none transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
  />

  {#if trailingAction}
    <div class="absolute right-2.5 flex items-center">
      {@render trailingAction()}
    </div>
  {/if}
</div>
