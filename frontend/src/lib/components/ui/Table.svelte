<script lang="ts">
  import type { Snippet } from 'svelte';

  export interface Column<T = any> {
    key: string;
    label: string;
    sortable?: boolean;
    align?: 'left' | 'center' | 'right';
    width?: string;
  }

  interface Props<T = any> {
    columns: Column<T>[];
    data: T[];
    sortKey?: string;
    sortDirection?: 'asc' | 'desc';
    striped?: boolean;
    compact?: boolean;
    pageSize?: number;
    emptyText?: string;
    class?: string;
    onRowClick?: (row: T) => void;
    onSort?: (key: string, direction: 'asc' | 'desc') => void;
    // Snippets
    cell?: Snippet<[{ item: T; column: Column<T>; value: any }]>;
    actions?: Snippet<[{ item: T }]>;
  }

  let {
    columns,
    data = [],
    sortKey = $bindable(''),
    sortDirection = $bindable<'asc' | 'desc'>('asc'),
    striped = false,
    compact = false,
    pageSize = 10,
    emptyText = 'No records found',
    class: customClass = '',
    onRowClick,
    onSort,
    cell,
    actions,
  }: Props = $props();

  let currentPage = $state(1);

  function handleHeaderClick(col: Column) {
    if (!col.sortable) return;
    if (sortKey === col.key) {
      sortDirection = sortDirection === 'asc' ? 'desc' : 'asc';
    } else {
      sortKey = col.key;
      sortDirection = 'asc';
    }
    onSort?.(sortKey, sortDirection);
  }

  const sortedData = $derived.by(() => {
    if (!sortKey) return data;
    return [...data].sort((a: any, b: any) => {
      const valA = a[sortKey];
      const valB = b[sortKey];
      if (valA === valB) return 0;
      if (valA == null) return 1;
      if (valB == null) return -1;
      
      const comparison = valA < valB ? -1 : 1;
      return sortDirection === 'asc' ? comparison : -comparison;
    });
  });

  const totalPages = $derived(Math.max(1, Math.ceil(sortedData.length / (pageSize || 10))));
  const paginatedData = $derived(
    pageSize > 0
      ? sortedData.slice((currentPage - 1) * pageSize, currentPage * pageSize)
      : sortedData
  );

  function prevPage() {
    if (currentPage > 1) currentPage--;
  }

  function nextPage() {
    if (currentPage < totalPages) currentPage++;
  }
</script>

<div class="w-full rounded-xl border border-neutral-200 dark:border-[#272732] bg-white dark:bg-[#0A0A0C] overflow-hidden flex flex-col font-sans {customClass}">
  <div class="overflow-x-auto w-full">
    <table class="w-full text-left text-xs border-collapse">
      <thead class="bg-neutral-50 dark:bg-[#121217] border-b border-neutral-200 dark:border-[#272732] text-neutral-600 dark:text-neutral-400 font-semibold uppercase tracking-wider text-[11px] select-none">
        <tr>
          {#each columns as col}
            <th
              class="py-3 px-4 transition-colors {col.sortable ? 'cursor-pointer hover:text-neutral-900 dark:hover:text-white' : ''} {col.align === 'center' ? 'text-center' : col.align === 'right' ? 'text-right' : 'text-left'}"
              style={col.width ? `width: ${col.width}` : ''}
              onclick={() => handleHeaderClick(col)}
            >
              <div class="inline-flex items-center gap-1.5 {col.align === 'center' ? 'justify-center' : col.align === 'right' ? 'justify-end' : 'justify-start'}">
                <span>{col.label}</span>
                {#if col.sortable}
                  <span class="inline-flex flex-col text-[8px] leading-[8px] {sortKey === col.key ? 'text-[var(--ca-brand)]' : 'text-neutral-400 dark:text-neutral-600'}">
                    {#if sortKey === col.key}
                      {sortDirection === 'asc' ? '▲' : '▼'}
                    {:else}
                      ▲▼
                    {/if}
                  </span>
                {/if}
              </div>
            </th>
          {/each}
          {#if actions}
            <th class="py-3 px-4 text-right w-24">Actions</th>
          {/if}
        </tr>
      </thead>
      <tbody class="divide-y divide-neutral-200 dark:divide-[#272732]/60 text-neutral-800 dark:text-neutral-200">
        {#if paginatedData.length === 0}
          <tr>
            <td
              colspan={columns.length + (actions ? 1 : 0)}
              class="py-8 text-center text-neutral-400 dark:text-neutral-500 italic"
            >
              {emptyText}
            </td>
          </tr>
        {:else}
          {#each paginatedData as row, idx (idx)}
            <tr
              class="transition-colors duration-100 {striped && idx % 2 === 1 ? 'bg-neutral-50/50 dark:bg-[#14141A]/50' : 'bg-transparent'} hover:bg-neutral-100/80 dark:hover:bg-[#181822]/80 {onRowClick ? 'cursor-pointer' : ''}"
              onclick={() => onRowClick?.(row)}
            >
              {#each columns as col}
                <td class="{compact ? 'py-2 px-3' : 'py-3 px-4'} {col.align === 'center' ? 'text-center' : col.align === 'right' ? 'text-right' : 'text-left'}">
                  {#if cell}
                    {@render cell({ item: row, column: col, value: row[col.key] })}
                  {:else}
                    <span class="truncate block">{row[col.key] ?? '—'}</span>
                  {/if}
                </td>
              {/each}
              {#if actions}
                <td class="{compact ? 'py-2 px-3' : 'py-3 px-4'} text-right whitespace-nowrap" onclick={(e) => e.stopPropagation()}>
                  {@render actions({ item: row })}
                </td>
              {/if}
            </tr>
          {/each}
        {/if}
      </tbody>
    </table>
  </div>

  <!-- Pagination Bar -->
  {#if pageSize > 0 && sortedData.length > 0}
    <div class="px-4 py-2.5 bg-neutral-50 dark:bg-[#121217] border-t border-neutral-200 dark:border-[#272732] flex items-center justify-between text-xs text-neutral-500 dark:text-neutral-400 select-none">
      <div>
        Showing <span class="font-medium text-neutral-800 dark:text-neutral-200">{(currentPage - 1) * pageSize + 1}</span> to <span class="font-medium text-neutral-800 dark:text-neutral-200">{Math.min(currentPage * pageSize, sortedData.length)}</span> of <span class="font-medium text-neutral-800 dark:text-neutral-200">{sortedData.length}</span> results
      </div>
      <div class="flex items-center gap-1.5">
        <button
          type="button"
          disabled={currentPage === 1}
          onclick={prevPage}
          class="px-2.5 py-1 rounded border border-neutral-200 dark:border-[#272732] bg-white dark:bg-[#18181F] text-neutral-700 dark:text-neutral-300 disabled:opacity-40 disabled:cursor-not-allowed hover:bg-neutral-100 dark:hover:bg-neutral-800 transition cursor-pointer"
        >
          Previous
        </button>
        <span class="px-2 font-mono text-[11px]">
          {currentPage} / {totalPages}
        </span>
        <button
          type="button"
          disabled={currentPage === totalPages}
          onclick={nextPage}
          class="px-2.5 py-1 rounded border border-neutral-200 dark:border-[#272732] bg-white dark:bg-[#18181F] text-neutral-700 dark:text-neutral-300 disabled:opacity-40 disabled:cursor-not-allowed hover:bg-neutral-100 dark:hover:bg-neutral-800 transition cursor-pointer"
        >
          Next
        </button>
      </div>
    </div>
  {/if}
</div>
