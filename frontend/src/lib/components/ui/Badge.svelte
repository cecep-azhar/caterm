<script context="module" lang="ts">
  export type BadgeVariant = 'primary' | 'secondary' | 'outline' | 'ghost' | 'brand' | 'neutral' | 'info' | 'success' | 'warning' | 'danger';
  export type BadgeSize = 'xs' | 'sm' | 'md';
</script>
<script lang="ts">
  import { Badge } from '@cecepazhar/caui';
</script>

<Badge {...$$props} />
