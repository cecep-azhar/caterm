import { invoke } from '@tauri-apps/api/core';

export interface SftpFileEntry {
  name: string;
  path: string;
  is_dir: boolean;
  size: number;
  mtime: number;
}

export function listRemoteDir(hostId: string, remotePath: string): Promise<SftpFileEntry[]> {
  return invoke('list_remote_dir', { hostId, remotePath });
}

export function readRemoteFile(hostId: string, remotePath: string): Promise<number[]> {
  return invoke('read_remote_file', { hostId, remotePath });
}

export function writeRemoteFile(hostId: string, remotePath: string, data: number[]): Promise<void> {
  return invoke('write_remote_file', { hostId, remotePath, data });
}

export function deleteRemoteFile(hostId: string, remotePath: string): Promise<void> {
  return invoke('delete_remote_file', { hostId, remotePath });
}