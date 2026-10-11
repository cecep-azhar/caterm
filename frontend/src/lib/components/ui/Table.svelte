<script module lang="ts">
  export interface Column<T = unknown> {
    key: string;
    label: string;
    sortable?: boolean;
    align?: 'left' | 'center' | 'right';
    width?: string;
  }
</script>
<script lang="ts">
  import { Table } from '@cecepazhar/caui';
  export type { Column };
  interface Props<T = unknown> { columns: Column<T>[]; data: T[]; sortKey?: string; sortDirection?: 'asc' | 'desc'; striped?: boolean; variant?: 'primary' | 'outline'; size?: 'sm' | 'md' | 'lg'; loading?: boolean; caption?: string; pageSize?: number; emptyText?: string; class?: string; onRowClick?: (row: T) => void; onSort?: (key: string, direction: 'asc' | 'desc') => void }
  let { ...rest }: Props = $props();
</script>

<Table {...rest} />
