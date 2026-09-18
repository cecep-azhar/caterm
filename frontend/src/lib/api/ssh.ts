// JS <-> Rust binding for SSH sessions. Backed by the placeholder engine in
// crates/caterm-core/src/ssh.rs today (no real socket yet, `write` just
// echoes) — the contract here is what Fase 2's real `russh`/`ssh2` transport
// will fill in without the frontend needing to change.
import { invoke } from '@tauri-apps/api/core';

export interface SshConnectRequest {
  hostId: string;
  address: string;
  port: number;
  username: string;
}

export interface SshSession {
  sessionId: string;
  hostId: string;
}

export function sshConnect(request: SshConnectRequest): Promise<SshSession> {
  return invoke('ssh_connect', { request });
}

export function sshWrite(sessionId: string, data: string): Promise<string> {
  return invoke('ssh_write', { sessionId, data });
}

export function sshResize(sessionId: string, cols: number, rows: number): Promise<void> {
  return invoke('ssh_resize', { sessionId, cols, rows });
}

export function sshDisconnect(sessionId: string): Promise<void> {
  return invoke('ssh_disconnect', { sessionId });
}
