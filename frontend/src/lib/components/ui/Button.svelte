<script context="module" lang="ts">
  export type ButtonVariant = 'primary' | 'secondary' | 'outline' | 'ghost' | 'brand' | 'neutral' | 'info' | 'success' | 'warning' | 'danger';
  export type ButtonSize = 'sm' | 'md' | 'lg' | 'icon';
</script>
<script lang="ts">
  import { Button } from '@cecepazhar/caui';
</script>

<Button {...$$props} />
