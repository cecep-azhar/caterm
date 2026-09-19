// JS <-> Rust binding for SSH sessions. Backed by the real `ssh2` engine in
// crates/caterm-core/src/ssh.rs — `sshConnect` only ever takes a saved host id, never raw
// connection details: the backend resolves the host record and its encrypted-at-rest
// credential itself (crate::store::load_host_for_connect) so plaintext never has to be
// re-sent over IPC on every connect.
import { invoke } from '@tauri-apps/api/core';

export interface SshSession {
  sessionId: string;
  hostId: string;
}

export function sshConnect(hostId: string): Promise<SshSession> {
  return invoke('ssh_connect', { hostId });
}

export function sshWrite(sessionId: string, data: string): Promise<string> {
  return invoke('ssh_write', { sessionId, data });
}

export function sshRead(sessionId: string): Promise<string> {
  return invoke('ssh_read', { sessionId });
}

export function sshResize(sessionId: string, cols: number, rows: number): Promise<void> {
  return invoke('ssh_resize', { sessionId, cols, rows });
}

export function sshDisconnect(sessionId: string): Promise<void> {
  return invoke('ssh_disconnect', { sessionId });
}
