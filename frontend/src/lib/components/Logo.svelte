<script lang="ts">
  export let size: number | string = 28;
  export let mode: 'light' | 'dark' | 'auto' = 'auto';
  export let className: string = '';
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 100 100"
  fill="currentColor"
  class="shrink-0 {mode === 'light' ? 'text-black' : mode === 'dark' ? 'text-white' : 'text-neutral-900 dark:text-white'} {className}"
  xmlns="http://www.w3.org/2000/svg"
>
  <!-- Left Segment -->
  <path
    fill-rule="evenodd"
    clip-rule="evenodd"
    d="M44 4L4 27V73L44 96V78L19 50L44 22V4ZM13 32.5L36 19V28.5L20 50L36 71.5V81L13 67.5V32.5Z"
  />
  <!-- Right Segment -->
  <path
    fill-rule="evenodd"
    clip-rule="evenodd"
    d="M56 4L96 27V73L56 96V78L81 50L56 22V4ZM87 32.5L64 19V28.5L80 50L64 71.5V81L87 67.5V32.5Z"
  />
</svg>
