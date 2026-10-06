<script lang="ts">
  import { onMount } from 'svelte';
  import {
    listScheduledTasks,
    saveScheduledTask,
    deleteScheduledTask,
    triggerTaskRunNow,
    getTaskExecutionLogs,
    type ScheduledTaskRecord,
    type TaskExecutionLog,
    type TaskType
  } from '$lib/api/tasks';
  import { listHosts, type HostRecord } from '$lib/api/hosts';
  import { t } from '$lib/i18n/index.svelte';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import { showToast, confirmModal } from '$lib/stores/uiNotifications.svelte';

  let tasks = $state<ScheduledTaskRecord[]>([]);
  let hosts = $state<HostRecord[]>([]);
  let isLoading = $state(false);
  let searchQuery = $state('');
  let filterType = $state<string>('all');

  // Modal State: Create / Edit Task
  let isAddModalOpen = $state(false);
  let editingId = $state<string | null>(null);
  let formName = $state('');
  let formDescription = $state('');
  let formTaskType = $state<TaskType>('ssh_command');
  let formHostId = $state('');
  let formScheduleExpr = $state('0 2 * * *');
  let formCommandScript = $state('');
  let formTimeoutSeconds = $state(300);
  let formRemoteSrcPath = $state('');
  let formLocalDestDir = $state('');
  let formRemotePreCmd = $state('');
  let formRemotePostCmd = $state('');
  let formRetentionCount = $state(7);
  let formNotifySuccess = $state(false);
  let formNotifyFailure = $state(true);

  // Modal State: Log Inspector
  let isLogModalOpen = $state(false);
  let activeTaskForLogs = $state<ScheduledTaskRecord | null>(null);
  let activeTaskLogs = $state<TaskExecutionLog[]>([]);
  let isExecutingTaskId = $state<string | null>(null);

  async function loadData() {
    try {
      const [tList, hList] = await Promise.all([
        listScheduledTasks(),
        listHosts()
      ]);
      tasks = tList || [];
      hosts = hList || [];
    } catch (e) {
      console.warn('Failed to load tasks/hosts:', e);
    }
  }

  onMount(() => {
    loadData();
    const handleUpdate = () => { void loadData(); };
    window.addEventListener('caterm:tasks-updated', handleUpdate);
    return () => {
      window.removeEventListener('caterm:tasks-updated', handleUpdate);
    };
  });

  function hostName(id?: string) {
    if (!id) return 'Local PC';
    return hosts.find(h => h.id === id)?.label || id;
  }

  function openAddModal() {
    editingId = null;
    formName = '';
    formDescription = '';
    formTaskType = 'ssh_command';
    formHostId = hosts[0]?.id || '';
    formScheduleExpr = '0 2 * * *';
    formCommandScript = '#!/usr/bin/env bash\necho "Running maintenance..."\n';
    formTimeoutSeconds = 300;
    formRemoteSrcPath = '/var/backups/db.sql.gz';
    formLocalDestDir = '/home/cecepazhar/Backups';
    formRemotePreCmd = 'mysqldump -u root ... | gzip > /var/backups/db.sql.gz';
    formRemotePostCmd = 'rm -f /var/backups/db.sql.gz';
    formRetentionCount = 7;
    formNotifySuccess = false;
    formNotifyFailure = true;
    isAddModalOpen = true;
  }

  function openEditModal(task: ScheduledTaskRecord) {
    editingId = task.id;
    formName = task.name;
    formDescription = task.description || '';
    formTaskType = task.taskType;
    formHostId = task.hostId || '';
    formScheduleExpr = task.scheduleExpr;
    formCommandScript = task.commandScript || '';
    formTimeoutSeconds = task.timeoutSeconds || 300;
    formRemoteSrcPath = task.remoteSrcPath || '';
    formLocalDestDir = task.localDestDir || '';
    formRemotePreCmd = task.remotePreCmd || '';
    formRemotePostCmd = task.remotePostCmd || '';
    formRetentionCount = task.retentionCount || 7;
    formNotifySuccess = task.notifyOnSuccess;
    formNotifyFailure = task.notifyOnFailure;
    isAddModalOpen = true;
  }

  async function handleSaveTask() {
    if (!formName.trim()) {
      showToast('Nama tugas wajib diisi', 'error');
      return;
    }

    isLoading = true;
    try {
      await saveScheduledTask({
        id: editingId ?? undefined,
        name: formName.trim(),
        description: formDescription.trim() || undefined,
        taskType: formTaskType,
        hostId: formHostId || undefined,
        scheduleExpr: formScheduleExpr.trim(),
        isEnabled: true,
        commandScript: formTaskType === 'ssh_command' || formTaskType === 'local_script' ? formCommandScript : undefined,
        timeoutSeconds: formTimeoutSeconds,
        remoteSrcPath: formTaskType === 'sftp_backup' ? formRemoteSrcPath : undefined,
        localDestDir: formTaskType === 'sftp_backup' ? formLocalDestDir : undefined,
        remotePreCmd: formTaskType === 'sftp_backup' ? formRemotePreCmd : undefined,
        remotePostCmd: formTaskType === 'sftp_backup' ? formRemotePostCmd : undefined,
        retentionCount: formRetentionCount,
        notifyOnSuccess: formNotifySuccess,
        notifyOnFailure: formNotifyFailure
      });

      isAddModalOpen = false;
      showToast(editingId ? 'Tugas diperbarui!' : 'Tugas berhasil dijadwalkan!', 'success');
      await loadData();
    } catch (e: any) {
      showToast(String(e), 'error');
    } finally {
      isLoading = false;
    }
  }

  async function handleDeleteTask(task: ScheduledTaskRecord) {
    const confirmed = await confirmModal(
      `Hapus tugas terjadwal "${task.name}"? Riwayat log eksekusi akan ikut dibersihkan.`,
      'Hapus Tugas Terjadwal',
      true,
      'Hapus',
      'Batal'
    );
    if (confirmed) {
      try {
        await deleteScheduledTask(task.id);
        showToast(`Tugas "${task.name}" dihapus!`, 'success');
        await loadData();
      } catch (e: any) {
        showToast(String(e), 'error');
      }
    }
  }

  async function handleRunNow(task: ScheduledTaskRecord) {
    isExecutingTaskId = task.id;
    try {
      const log = await triggerTaskRunNow(task.id);
      showToast(`Tugas "${task.name}" selesai dieksekusi! (${log.status})`, log.status === 'success' ? 'success' : 'error');
      await loadData();
    } catch (e: any) {
      showToast(`Gagal menjalankan tugas: ${e}`, 'error');
    } finally {
      isExecutingTaskId = null;
    }
  }

  async function openLogModal(task: ScheduledTaskRecord) {
    activeTaskForLogs = task;
    try {
      activeTaskLogs = await getTaskExecutionLogs(task.id);
    } catch {
      activeTaskLogs = [];
    }
    isLogModalOpen = true;
  }

  const filteredTasks = $derived.by(() => {
    let list = tasks;
    if (filterType !== 'all') {
      list = list.filter(t => t.taskType === filterType);
    }
    const q = searchQuery.toLowerCase().trim();
    if (!q) return list;
    return list.filter(t => t.name.toLowerCase().includes(q) || (t.description && t.description.toLowerCase().includes(q)));
  });
</script>

<svelte:head>
  <title>Scheduled Tasks & SFTP Auto-Backup · CATerm</title>
</svelte:head>

<div class="max-w-5xl mx-auto space-y-6">
  <PageHeader
    icon={['M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z']}
    accent="indigo"
    title={t('tasks.title')}
    subtitle={t('tasks.subtitle')}
  >
    {#snippet actions()}
      <button
        type="button"
        onclick={openAddModal}
        class="inline-flex items-center justify-center gap-2 px-4 py-2 rounded-xl text-xs font-semibold bg-sky-600 hover:bg-sky-500 text-white shadow-md shadow-sky-600/20 transition-all shrink-0"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
        </svg>
        {t('tasks.newTask')}
      </button>
    {/snippet}
  </PageHeader>

  <!-- Filter & Status Bar -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 border-b border-neutral-200 dark:border-neutral-800 pb-3">
    <div class="flex items-center gap-1.5 overflow-x-auto text-xs">
      <button
        type="button"
        onclick={() => (filterType = 'all')}
        class="px-3 py-1.5 rounded-lg font-medium transition-colors {filterType === 'all' ? 'bg-neutral-900 text-white dark:bg-white dark:text-neutral-900' : 'text-neutral-600 dark:text-neutral-400 hover:bg-neutral-100 dark:hover:bg-neutral-800'}"
      >
        {t('tasks.allTasks', { count: tasks.length })}
      </button>
      <button
        type="button"
        onclick={() => (filterType = 'sftp_backup')}
        class="px-3 py-1.5 rounded-lg font-medium transition-colors {filterType === 'sftp_backup' ? 'bg-indigo-600 text-white' : 'text-neutral-600 dark:text-neutral-400 hover:bg-neutral-100 dark:hover:bg-neutral-800'}"
      >
        📦 {t('tasks.sftpBackup')}
      </button>
      <button
        type="button"
        onclick={() => (filterType = 'ssh_command')}
        class="px-3 py-1.5 rounded-lg font-medium transition-colors {filterType === 'ssh_command' ? 'bg-sky-600 text-white' : 'text-neutral-600 dark:text-neutral-400 hover:bg-neutral-100 dark:hover:bg-neutral-800'}"
      >
        ⚡ {t('tasks.sshPlaybook')}
      </button>
    </div>

    <!-- Search Input -->
    <div class="relative w-full sm:w-64">
      <input
        type="text"
        bind:value={searchQuery}
        placeholder={t('tasks.searchPlaceholder')}
        class="w-full pl-8 pr-3 py-1.5 text-xs rounded-xl bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 text-neutral-900 dark:text-white placeholder-neutral-400 focus:outline-none focus:border-sky-500"
      />
      <svg class="w-3.5 h-3.5 text-neutral-400 absolute left-2.5 top-2.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
      </svg>
    </div>
  </div>

  <!-- Tasks Grid -->
  {#if filteredTasks.length === 0}
    <div class="text-center py-16 px-4 bg-white dark:bg-neutral-900/40 rounded-2xl border border-neutral-200 dark:border-neutral-800 space-y-3">
      <div class="w-12 h-12 rounded-full bg-indigo-500/10 text-indigo-500 flex items-center justify-center mx-auto border border-indigo-500/20">
        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
      </div>
      <h3 class="text-sm font-semibold text-neutral-900 dark:text-white">{t('tasks.emptyTitle')}</h3>
      <p class="text-xs text-neutral-500 max-w-sm mx-auto">
        {t('tasks.emptyBody')}
      </p>
      <button
        type="button"
        onclick={openAddModal}
        class="mt-2 inline-flex items-center gap-1.5 px-3.5 py-1.5 rounded-lg text-xs font-semibold bg-sky-600 hover:bg-sky-500 text-white transition-colors"
      >
        {t('tasks.scheduleFirst')}
      </button>
    </div>
  {:else}
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      {#each filteredTasks as task (task.id)}
        <div class="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-2xl p-5 flex flex-col justify-between hover:border-neutral-300 dark:hover:border-neutral-700 transition-all shadow-xs group">
          <div class="space-y-3">
            <!-- Header -->
            <div class="flex items-start justify-between gap-2">
              <div>
                <div class="flex items-center gap-2">
                  <span class="text-xs px-2 py-0.5 rounded-md font-bold uppercase tracking-wider {task.taskType === 'sftp_backup' ? 'bg-indigo-500/15 text-indigo-700 dark:text-indigo-300 border border-indigo-500/30' : 'bg-sky-500/15 text-sky-700 dark:text-sky-300 border border-sky-500/30'}">
                    {task.taskType === 'sftp_backup' ? '📦 SFTP Backup' : '⚡ SSH Playbook'}
                  </span>
                  <span class="text-xs text-neutral-500 font-mono">
                    ⏰ {task.scheduleExpr}
                  </span>
                </div>
                <h3 class="font-bold text-neutral-900 dark:text-white text-base mt-1.5">{task.name}</h3>
                {#if task.description}
                  <p class="text-xs text-neutral-500 line-clamp-1 mt-0.5">{task.description}</p>
                {/if}
              </div>

              <!-- Status Badge -->
              <span class="px-2 py-0.5 rounded-full text-[10px] font-semibold {task.isEnabled ? 'bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border border-emerald-500/20' : 'bg-neutral-200 dark:bg-neutral-800 text-neutral-500'}">
                {task.isEnabled ? 'Active' : 'Paused'}
              </span>
            </div>

            <!-- Details Box -->
            <div class="bg-neutral-50 dark:bg-neutral-950/60 rounded-xl p-3 border border-neutral-100 dark:border-neutral-800/60 text-xs space-y-1.5 font-mono">
              <div class="flex items-center justify-between text-neutral-600 dark:text-neutral-400">
                <span class="text-[10px] uppercase font-sans font-semibold text-neutral-400">Target Host:</span>
                <span class="font-medium text-neutral-900 dark:text-white">{hostName(task.hostId)}</span>
              </div>
              {#if task.taskType === 'sftp_backup'}
                <div class="flex items-center justify-between text-neutral-600 dark:text-neutral-400 truncate">
                  <span class="text-[10px] uppercase font-sans font-semibold text-neutral-400 shrink-0">Remote Path:</span>
                  <span class="truncate ml-2 text-indigo-600 dark:text-indigo-400">{task.remoteSrcPath}</span>
                </div>
                <div class="flex items-center justify-between text-neutral-600 dark:text-neutral-400">
                  <span class="text-[10px] uppercase font-sans font-semibold text-neutral-400">Retensi Lokal:</span>
                  <span>{task.retentionCount} file terakhir</span>
                </div>
              {/if}
            </div>
          </div>

          <!-- Footer Actions -->
          <div class="mt-4 pt-3 border-t border-neutral-100 dark:border-neutral-800/80 flex items-center justify-between text-xs">
            <button
              type="button"
              onclick={() => openLogModal(task)}
              class="text-neutral-500 hover:text-sky-500 transition-colors flex items-center gap-1 font-medium"
            >
              📜 Riwayat Log
            </button>

            <div class="flex items-center gap-1.5">
              <button
                type="button"
                onclick={() => handleRunNow(task)}
                disabled={isExecutingTaskId === task.id}
                class="px-2.5 py-1 bg-sky-500/10 hover:bg-sky-600 hover:text-white text-sky-600 dark:text-sky-400 rounded-lg font-semibold transition-colors disabled:opacity-50"
              >
                {isExecutingTaskId === task.id ? 'Running...' : '⚡ Run Now'}
              </button>
              <button
                type="button"
                onclick={() => openEditModal(task)}
                class="px-2.5 py-1 bg-neutral-100 dark:bg-neutral-800 hover:bg-neutral-200 dark:hover:bg-neutral-700 text-neutral-700 dark:text-neutral-300 rounded-lg transition-colors"
              >
                Edit
              </button>
              <button
                type="button"
                onclick={() => handleDeleteTask(task)}
                class="px-2.5 py-1 bg-neutral-100 dark:bg-neutral-800 hover:bg-rose-600 hover:text-white text-neutral-700 dark:text-neutral-300 rounded-lg transition-colors"
              >
                Hapus
              </button>
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<!-- Modal Create / Edit Task -->
{#if isAddModalOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 dark:bg-black/80 backdrop-blur-xs">
    <div
      role="dialog"
      aria-modal="true"
      class="w-full max-w-lg max-h-[90vh] overflow-y-auto bg-white dark:bg-[#141414] border border-neutral-200 dark:border-neutral-800 rounded-2xl p-6 space-y-4 shadow-2xl animate-in fade-in zoom-in-95 duration-150 text-xs"
    >
      <div class="flex items-center justify-between border-b border-neutral-200 dark:border-neutral-800 pb-3">
        <h3 class="text-base font-bold text-neutral-900 dark:text-white">
          {editingId ? 'Edit Tugas Terjadwal' : 'Tambah Tugas Terjadwal Baru'}
        </h3>
        <button
          type="button"
          onclick={() => (isAddModalOpen = false)}
          class="text-neutral-400 hover:text-neutral-600 dark:hover:text-neutral-200 text-lg font-bold"
        >
          ×
        </button>
      </div>

      <div class="space-y-3">
        <div>
          <label class="block font-medium text-neutral-700 dark:text-neutral-300 mb-1">Nama Tugas *</label>
          <input
            type="text"
            bind:value={formName}
            placeholder="misal: Daily Backup Database VPS Hostinger"
            class="w-full px-3 py-2 rounded-lg bg-neutral-50 dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500"
          />
        </div>

        <div>
          <label class="block font-medium text-neutral-700 dark:text-neutral-300 mb-1">Tipe Tugas</label>
          <div class="grid grid-cols-2 gap-2">
            <button
              type="button"
              onclick={() => (formTaskType = 'sftp_backup')}
              class="p-2.5 rounded-lg border text-left font-medium transition-all {formTaskType === 'sftp_backup' ? 'bg-indigo-500/10 border-indigo-500 text-indigo-700 dark:text-indigo-300' : 'bg-neutral-50 dark:bg-neutral-900 border-neutral-300 dark:border-neutral-800 text-neutral-700 dark:text-neutral-400'}"
            >
              📦 SFTP Auto-Backup
            </button>
            <button
              type="button"
              onclick={() => (formTaskType = 'ssh_command')}
              class="p-2.5 rounded-lg border text-left font-medium transition-all {formTaskType === 'ssh_command' ? 'bg-sky-500/10 border-sky-500 text-sky-700 dark:text-sky-300' : 'bg-neutral-50 dark:bg-neutral-900 border-neutral-300 dark:border-neutral-800 text-neutral-700 dark:text-neutral-400'}"
            >
              ⚡ SSH Script Playbook
            </button>
          </div>
        </div>

        <div>
          <label class="block font-medium text-neutral-700 dark:text-neutral-300 mb-1">Target Server (Host)</label>
          <select
            bind:value={formHostId}
            class="w-full px-3 py-2 rounded-lg bg-neutral-50 dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500"
          >
            {#each hosts as h}
              <option value={h.id}>{h.label} ({h.address})</option>
            {/each}
          </select>
        </div>

        <div>
          <label class="block font-medium text-neutral-700 dark:text-neutral-300 mb-1">Jadwal Eksekusi (Cron / Interval) *</label>
          <input
            type="text"
            bind:value={formScheduleExpr}
            placeholder="misal: 0 2 * * * (Jam 02:00 WIB) atau every 6h"
            class="w-full px-3 py-2 font-mono rounded-lg bg-neutral-50 dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500"
          />
        </div>

        {#if formTaskType === 'sftp_backup'}
          <div>
            <label class="block font-medium text-neutral-700 dark:text-neutral-300 mb-1">Remote File Path *</label>
            <input
              type="text"
              bind:value={formRemoteSrcPath}
              placeholder="/var/backups/db.sql.gz"
              class="w-full px-3 py-2 font-mono rounded-lg bg-neutral-50 dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500"
            />
          </div>

          <div>
            <label class="block font-medium text-neutral-700 dark:text-neutral-300 mb-1">Folder Penyimpanan Lokal *</label>
            <input
              type="text"
              bind:value={formLocalDestDir}
              placeholder="/home/cecepazhar/Backups"
              class="w-full px-3 py-2 font-mono rounded-lg bg-neutral-50 dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500"
            />
          </div>

          <div>
            <label class="block font-medium text-neutral-700 dark:text-neutral-300 mb-1">Pre-command Dump di Server (Opsional)</label>
            <input
              type="text"
              bind:value={formRemotePreCmd}
              placeholder="mysqldump -u root mydb | gzip > /var/backups/db.sql.gz"
              class="w-full px-3 py-2 font-mono rounded-lg bg-neutral-50 dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500"
            />
          </div>
        {:else}
          <div>
            <label class="block font-medium text-neutral-700 dark:text-neutral-300 mb-1">Bash Script Perintah *</label>
            <textarea
              bind:value={formCommandScript}
              rows="5"
              placeholder="#!/usr/bin/env bash&#10;docker system prune -f&#10;"
              class="w-full px-3 py-2 font-mono rounded-lg bg-neutral-50 dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500"
            ></textarea>
          </div>
        {/if}
      </div>

      <div class="flex justify-end gap-2.5 pt-4 border-t border-neutral-200 dark:border-neutral-800">
        <button
          type="button"
          onclick={() => (isAddModalOpen = false)}
          class="px-4 py-2 rounded-lg bg-neutral-100 dark:bg-neutral-800 hover:bg-neutral-200 dark:hover:bg-neutral-700 text-neutral-700 dark:text-neutral-300 font-semibold"
        >
          Batal
        </button>
        <button
          type="button"
          onclick={handleSaveTask}
          disabled={isLoading}
          class="px-4 py-2 rounded-lg bg-sky-600 hover:bg-sky-500 text-white font-semibold shadow-md shadow-sky-600/20 disabled:opacity-50"
        >
          {isLoading ? 'Menyimpan...' : 'Simpan Jadwal'}
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Modal Log Inspector -->
{#if isLogModalOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 dark:bg-black/80 backdrop-blur-xs">
    <div
      role="dialog"
      aria-modal="true"
      class="w-full max-w-2xl max-h-[85vh] overflow-y-auto bg-white dark:bg-[#141414] border border-neutral-200 dark:border-neutral-800 rounded-2xl p-6 space-y-4 shadow-2xl animate-in fade-in zoom-in-95 duration-150 text-xs"
    >
      <div class="flex items-center justify-between border-b border-neutral-200 dark:border-neutral-800 pb-3">
        <h3 class="text-base font-bold text-neutral-900 dark:text-white">
          Riwayat Log: {activeTaskForLogs?.name}
        </h3>
        <button
          type="button"
          onclick={() => (isLogModalOpen = false)}
          class="text-neutral-400 hover:text-neutral-600 dark:hover:text-neutral-200 text-lg font-bold"
        >
          ×
        </button>
      </div>

      {#if activeTaskLogs.length === 0}
        <div class="text-center py-8 text-neutral-500">
          Belum ada riwayat eksekusi untuk tugas ini.
        </div>
      {:else}
        <div class="space-y-3">
          {#each activeTaskLogs as log}
            <div class="p-3 rounded-xl bg-neutral-950 border border-neutral-800 font-mono text-[11px] text-neutral-300 space-y-1">
              <div class="flex items-center justify-between text-neutral-400">
                <span>{log.startedAt}</span>
                <span class="px-2 py-0.5 rounded text-[10px] font-bold {log.status === 'success' ? 'bg-emerald-500/20 text-emerald-400' : 'bg-rose-500/20 text-rose-400'}">
                  {log.status.toUpperCase()} ({log.durationMs || 0}ms)
                </span>
              </div>
              {#if log.stdout}
                <div class="mt-2 text-neutral-200 whitespace-pre-wrap">{log.stdout}</div>
              {/if}
              {#if log.errorMessage}
                <div class="mt-1 text-rose-400">{log.errorMessage}</div>
              {/if}
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>
{/if}
