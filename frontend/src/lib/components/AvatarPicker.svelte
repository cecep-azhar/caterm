<script lang="ts">
  import { t } from '$lib/i18n/index.svelte';
  import ProfileAvatar, { AVATARS } from './ProfileAvatar.svelte';

  let { value = $bindable(), size = 40 }: { value: string; size?: number } = $props();
</script>

<div class="grid grid-cols-5 gap-2" role="radiogroup" aria-label={t('avatars.pickerLabel')}>
  {#each AVATARS as option (option.id)}
    <button
      type="button"
      role="radio"
      aria-checked={value === option.id}
      aria-label={t(`avatars.${option.id}`)}
      title={t(`avatars.${option.id}`)}
      onclick={() => (value = option.id)}
      class="no-drag justify-self-center flex items-center justify-center rounded-full p-0.5 ring-2 ring-offset-2 ring-offset-transparent transition-all {value === option.id ? 'ring-sky-500' : 'ring-transparent hover:ring-neutral-300 dark:hover:ring-neutral-700'}"
    >
      <ProfileAvatar avatar={option.id} {size} />
    </button>
  {/each}
</div>
