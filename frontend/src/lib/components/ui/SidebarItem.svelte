<script lang="ts">
  import { SidebarItem } from '@cecepazhar/caui';
  interface Props { label: string; active?: boolean; icon?: import('svelte').Snippet; badge?: string | number; href?: string; onclick?: (e: MouseEvent) => void; class?: string; [key: string]: any }
  let { ...rest }: Props = $props();
</script>

<SidebarItem {...rest} />
