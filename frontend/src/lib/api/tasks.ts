import { invoke } from '@tauri-apps/api/core';

export type TaskType = 'ssh_command' | 'sftp_backup' | 'local_script';
export type TaskStatus = 'success' | 'failed' | 'running' | 'timeout' | 'cancelled';

export interface ScheduledTaskRecord {
  id: string;
  name: string;
  description?: string;
  taskType: TaskType;
  hostId?: string;
  scheduleExpr: string;
  isEnabled: boolean;
  commandScript?: string;
  timeoutSeconds: number;
  remoteSrcPath?: string;
  localDestDir?: string;
  remotePreCmd?: string;
  remotePostCmd?: string;
  retentionCount: number;
  notifyOnSuccess: boolean;
  notifyOnFailure: boolean;
  lastRunAt?: string;
  lastStatus?: TaskStatus;
  nextRunAt?: string;
  createdAt: string;
  updatedAt: string;
}

export interface TaskInput {
  id?: string;
  name: string;
  description?: string;
  taskType: TaskType;
  hostId?: string;
  scheduleExpr: string;
  isEnabled: boolean;
  commandScript?: string;
  timeoutSeconds?: number;
  remoteSrcPath?: string;
  localDestDir?: string;
  remotePreCmd?: string;
  remotePostCmd?: string;
  retentionCount?: number;
  notifyOnSuccess?: boolean;
  notifyOnFailure?: boolean;
}

export interface TaskExecutionLog {
  id: string;
  taskId: string;
  startedAt: string;
  finishedAt?: string;
  durationMs?: number;
  exitCode?: number;
  status: TaskStatus;
  stdout?: string;
  stderr?: string;
  bytesTransferred?: number;
  errorMessage?: string;
}

export async function listScheduledTasks(): Promise<ScheduledTaskRecord[]> {
  return invoke('list_scheduled_tasks');
}

export async function saveScheduledTask(input: TaskInput): Promise<ScheduledTaskRecord> {
  return invoke('save_scheduled_task', { input });
}

export async function deleteScheduledTask(id: string): Promise<void> {
  return invoke('delete_scheduled_task', { id });
}

export async function triggerTaskRunNow(taskId: string): Promise<TaskExecutionLog> {
  return invoke('trigger_task_run_now', { taskId });
}

export async function getTaskExecutionLogs(taskId: string): Promise<TaskExecutionLog[]> {
  return invoke('get_task_execution_logs', { taskId });
}
