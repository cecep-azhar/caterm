<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';

  onMount(() => {
    const qHost = page.url.searchParams.get('host');
    if (qHost) {
      goto(`/devops?host=${qHost}`, { replaceState: true });
    } else {
      goto('/devops', { replaceState: true });
    }
  });
</script>

<div class="p-8 text-center text-xs text-neutral-400">
  Redirecting to DevOps Lab...
</div>
