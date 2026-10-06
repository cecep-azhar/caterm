<script lang="ts">
  import { onMount } from 'svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { getTaskExecutionLogs, type TaskExecutionLog, type ScheduledTaskRecord } from '$lib/api/tasks';

  let {
    task,
    onClose
  }: {
    task: ScheduledTaskRecord;
    onClose: () => void;
  } = $props();

  let logs = $state<TaskExecutionLog[]>([]);
  let selectedLog = $state<TaskExecutionLog | null>(null);
  let isLoading = $state(true);
  let errorMsg = $state('');

  async function loadLogs() {
    isLoading = true;
    errorMsg = '';
    try {
      logs = await getTaskExecutionLogs(task.id);
      if (logs.length > 0) {
        selectedLog = logs[0];
      }
    } catch (e: any) {
      errorMsg = e?.message || String(e);
    } finally {
      isLoading = false;
    }
  }

  onMount(loadLogs);

  function formatBytes(bytes?: number): string {
    if (bytes === undefined || bytes === null || bytes === 0) return '0 B';
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    let val = bytes;
    let unitIdx = 0;
    while (val >= 1024 && unitIdx < units.length - 1) {
      val /= 1024;
      unitIdx++;
    }
    return `${val.toFixed(1)} ${units[unitIdx]}`;
  }

  function formatDuration(ms?: number): string {
    if (ms === undefined || ms === null) return '-';
    if (ms < 1000) return `${ms}ms`;
    const sec = (ms / 1000).toFixed(1);
    return `${sec}s`;
  }

  function getStatusBadge(status: string) {
    switch (status) {
      case 'success':
        return { bg: 'bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border-emerald-500/30', label: 'Success' };
      case 'failed':
        return { bg: 'bg-rose-500/10 text-rose-600 dark:text-rose-400 border-rose-500/30', label: 'Failed' };
      case 'timeout':
        return { bg: 'bg-amber-500/10 text-amber-600 dark:text-amber-400 border-amber-500/30', label: 'Timeout' };
      case 'running':
        return { bg: 'bg-sky-500/10 text-sky-600 dark:text-sky-400 border-sky-500/30 animate-pulse', label: 'Running' };
      default:
        return { bg: 'bg-neutral-500/10 text-neutral-500 border-neutral-500/30', label: status };
    }
  }
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && onClose()} />

<div class="fixed inset-0 z-[100] flex items-center justify-center p-4">
  <button
    type="button"
    class="fixed inset-0 bg-black/50 dark:bg-black/70 backdrop-blur-sm cursor-default"
    aria-label={t('common.close')}
    onclick={onClose}
  ></button>

  <div
    role="dialog"
    aria-modal="true"
    aria-labelledby="log-modal-title"
    class="no-drag relative w-full max-w-4xl rounded-2xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-[#141414] shadow-2xl overflow-hidden flex flex-col h-[85vh]"
  >
    <!-- Modal Header -->
    <div class="flex items-center justify-between px-6 py-4 border-b border-neutral-200 dark:border-neutral-800 bg-neutral-50/50 dark:bg-neutral-900/40 shrink-0">
      <div class="flex items-center gap-3 min-w-0">
        <div class="w-9 h-9 rounded-xl bg-purple-500/10 dark:bg-purple-500/20 border border-purple-500/30 flex items-center justify-center text-purple-600 dark:text-purple-400 shrink-0">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
        </div>
        <div class="min-w-0">
          <h2 id="log-modal-title" class="text-base font-bold text-neutral-900 dark:text-white truncate">
            {t('tasks.executionLogs')}: <span class="font-normal text-neutral-600 dark:text-neutral-300">{task.name}</span>
          </h2>
          <p class="text-xs text-neutral-500 dark:text-neutral-400 truncate">
            {task.task_type === 'ssh_command' ? 'SSH Script Playbook' : 'SFTP Pull Auto-Backup'} · {task.schedule_expr}
          </p>
        </div>
      </div>
      <div class="flex items-center gap-2">
        <button
          type="button"
          onclick={loadLogs}
          class="p-1.5 rounded-lg text-neutral-400 hover:text-neutral-900 dark:hover:text-white hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors"
          title={t('common.refresh')}
          aria-label={t('common.refresh')}
        >
          <svg class="w-4 h-4 {isLoading ? 'animate-spin text-purple-500' : ''}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
        </button>
        <button
          type="button"
          onclick={onClose}
          class="p-1.5 rounded-lg text-neutral-400 hover:text-neutral-900 dark:hover:text-white hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors"
          aria-label={t('common.close')}
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>
        </button>
      </div>
    </div>

    <!-- Main Container: 2-column log inspector -->
    <div class="flex flex-1 min-h-0 divide-x divide-neutral-200 dark:divide-neutral-800">
      <!-- Left sidebar: Run history list -->
      <div class="w-1/3 min-w-[240px] max-w-[320px] overflow-y-auto bg-neutral-50/50 dark:bg-neutral-900/20 divide-y divide-neutral-200/60 dark:divide-neutral-800/60">
        {#if isLoading && logs.length === 0}
          <div class="p-6 text-center text-xs text-neutral-400">
            <svg class="w-5 h-5 animate-spin mx-auto mb-2 text-purple-500" fill="none" viewBox="0 0 24 24"><circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle><path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path></svg>
            {t('common.loading')}
          </div>
        {:else if errorMsg}
          <div class="p-4 text-xs text-rose-500">
            {errorMsg}
          </div>
        {:else if logs.length === 0}
          <div class="p-6 text-center text-xs text-neutral-400">
            {t('tasks.noLogsYet')}
          </div>
        {:else}
          {#each logs as log}
            {@const badge = getStatusBadge(log.status)}
            <button
              type="button"
              onclick={() => (selectedLog = log)}
              class={`w-full p-3 text-left transition flex flex-col gap-1.5 ${
                selectedLog?.id === log.id
                  ? 'bg-purple-500/10 dark:bg-purple-500/20 border-l-2 border-purple-500 text-neutral-900 dark:text-white'
                  : 'hover:bg-neutral-100 dark:hover:bg-neutral-900/60 text-neutral-600 dark:text-neutral-400'
              }`}
            >
              <div class="flex items-center justify-between gap-1">
                <span class={`text-[10px] px-1.5 py-0.5 rounded-full font-semibold border ${badge.bg}`}>
                  {badge.label}
                </span>
                <span class="text-[10px] font-mono text-neutral-400">
                  {formatDuration(log.duration_ms)}
                </span>
              </div>
              <div class="text-xs font-mono font-medium truncate text-neutral-900 dark:text-neutral-200">
                {log.started_at}
              </div>
              {#if log.bytes_transferred}
                <div class="text-[10px] text-purple-600 dark:text-purple-400">
                  Transferred: {formatBytes(log.bytes_transferred)}
                </div>
              {/if}
            </button>
          {/each}
        {/if}
      </div>

      <!-- Right area: Detailed Log & Output View (Dark Terminal Style) -->
      <div class="flex-1 flex flex-col min-w-0 bg-[#0c0d10] text-neutral-200 font-mono text-xs">
        {#if selectedLog}
          <!-- Execution summary header -->
          <div class="px-5 py-3 bg-[#13151b] border-b border-neutral-800/80 flex flex-wrap items-center justify-between gap-3 text-[11px] shrink-0">
            <div class="flex items-center gap-3">
              <div>
                <span class="text-neutral-400">Started:</span> <span class="text-neutral-200">{selectedLog.started_at}</span>
              </div>
              {#if selectedLog.finished_at}
                <div>
                  <span class="text-neutral-400">Duration:</span> <span class="text-emerald-400">{formatDuration(selectedLog.duration_ms)}</span>
                </div>
              {/if}
            </div>

            <div class="flex items-center gap-3">
              {#if selectedLog.exit_code !== undefined && selectedLog.exit_code !== null}
                <div>
                  <span class="text-neutral-400">Exit Code:</span>
                  <span class={selectedLog.exit_code === 0 ? 'text-emerald-400 font-bold' : 'text-rose-400 font-bold'}>
                    {selectedLog.exit_code}
                  </span>
                </div>
              {/if}
              {#if selectedLog.bytes_transferred}
                <div>
                  <span class="text-neutral-400">Size:</span> <span class="text-purple-400">{formatBytes(selectedLog.bytes_transferred)}</span>
                </div>
              {/if}
            </div>
          </div>

          <!-- Console Terminal Output Window -->
          <div class="flex-1 p-5 overflow-y-auto space-y-4 font-mono select-text selection:bg-purple-800">
            {#if selectedLog.error_message}
              <div class="p-3 rounded-lg bg-rose-500/10 border border-rose-500/30 text-rose-300 space-y-1">
                <div class="text-[11px] font-bold uppercase tracking-wider text-rose-400">Error Details:</div>
                <div class="whitespace-pre-wrap text-xs">{selectedLog.error_message}</div>
              </div>
            {/if}

            {#if selectedLog.stdout}
              <div class="space-y-1">
                <div class="text-[10px] text-neutral-500 uppercase tracking-wider">── Standard Output (stdout) ──</div>
                <pre class="text-emerald-400 whitespace-pre-wrap leading-relaxed text-xs">{selectedLog.stdout}</pre>
              </div>
            {/if}

            {#if selectedLog.stderr}
              <div class="space-y-1">
                <div class="text-[10px] text-amber-500 uppercase tracking-wider">── Standard Error (stderr) ──</div>
                <pre class="text-rose-400 whitespace-pre-wrap leading-relaxed text-xs">{selectedLog.stderr}</pre>
              </div>
            {/if}

            {#if !selectedLog.stdout && !selectedLog.stderr && !selectedLog.error_message}
              <div class="text-neutral-500 italic py-8 text-center">
                (No console output recorded for this run)
              </div>
            {/if}
          </div>
        {:else}
          <div class="flex-1 flex items-center justify-center text-neutral-500 text-xs">
            {t('tasks.selectLogRun')}
          </div>
        {/if}
      </div>
    </div>
  </div>
</div>
