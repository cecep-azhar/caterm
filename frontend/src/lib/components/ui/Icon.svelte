<script module lang="ts">
  export type IconName = 'terminal' | 'server' | 'cpu' | 'shield' | 'sparkles' | 'settings' | 'code' | 'folder' | 'file' | 'search' | 'chevron-down' | 'chevron-right' | 'check' | 'x' | 'refresh' | 'user' | 'globe' | 'moon' | 'sun' | 'palette' | 'zap' | 'database' | 'info' | 'alert' | 'lock' | 'copy' | 'logout' | 'calendar' | 'upload' | 'download' | 'plus' | 'minus' | 'chevron-left' | 'chevron-up' | 'trash' | 'edit' | 'filter' | 'loader';
</script>
<script lang="ts">
  import { Icon } from '@cecepazhar/caui';
  export type { IconName };
  interface Props { name: IconName; size?: number | string; class?: string }
  let { ...rest }: Props = $props();
</script>

<Icon {...rest} />
