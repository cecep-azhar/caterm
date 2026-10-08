import { invoke } from '@tauri-apps/api/core';
import type { SftpFileEntry } from './sftp';

export interface VfsFileEntry extends SftpFileEntry {}

export function vfsListFiles(hostId: string, remotePath: string): Promise<VfsFileEntry[]> {
  return invoke('vfs_list_files', { hostId, remotePath });
}

export function vfsReadFile(hostId: string, remotePath: string): Promise<number[]> {
  return invoke('vfs_read_file', { hostId, remotePath });
}

export function vfsWriteFile(hostId: string, remotePath: string, data: number[]): Promise<void> {
  return invoke('vfs_write_file', { hostId, remotePath, data });
}

export function vfsCreateDir(hostId: string, remotePath: string): Promise<void> {
  return invoke('vfs_create_dir', { hostId, remotePath });
}

export function vfsDeleteFile(
  hostId: string,
  remotePath: string,
  isDir = false,
  recursive = false
): Promise<void> {
  return invoke('vfs_delete_file', { hostId, remotePath, isDir, recursive });
}
