<script lang="ts">
  import ProfileAvatar, { AVATARS } from './ProfileAvatar.svelte';

  let { value = $bindable(), size = 40 }: { value: string; size?: number } = $props();
</script>

<div class="grid grid-cols-5 gap-2" role="radiogroup" aria-label="Profile picture">
  {#each AVATARS as option (option.id)}
    <button
      type="button"
      role="radio"
      aria-checked={value === option.id}
      aria-label={option.label}
      title={option.label}
      onclick={() => (value = option.id)}
      class="no-drag justify-self-center flex items-center justify-center rounded-full p-0.5 ring-2 ring-offset-2 ring-offset-transparent transition-all {value === option.id ? 'ring-sky-500' : 'ring-transparent hover:ring-neutral-300 dark:hover:ring-neutral-700'}"
    >
      <ProfileAvatar avatar={option.id} {size} />
    </button>
  {/each}
</div>
