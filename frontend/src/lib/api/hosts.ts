// JS <-> Rust binding for host persistence. Mirrors `caterm_core::store` 1:1 —
// see crates/caterm-core/src/store.rs for the source of truth on shapes and
// error codes. No secret material (passwords, key bytes) ever crosses this
// boundary; that is zero-knowledge vault territory for a later phase.
import { invoke } from '@tauri-apps/api/core';

export type AuthMethod = { type: 'password' } | { type: 'key'; path: string } | { type: 'keyId'; id: string };

export interface HostRecord {
  id: string;
  label: string;
  address: string;
  port: number;
  username: string;
  authMethod: AuthMethod;
  tags: string[];
  createdAt: number;
  updatedAt: number;
  /** Whether a password/passphrase is already stored for this host (never the value itself). */
  hasSecret: boolean;
}

export interface HostInput {
  id?: string;
  label: string;
  address: string;
  port: number;
  username: string;
  authMethod: AuthMethod;
  tags: string[];
  /** Write-only. Omit to leave the stored secret untouched, "" to clear it, or a new
   * password/passphrase to (re)encrypt and store it. Never comes back out via HostRecord. */
  secret?: string;
}

export function listHosts(): Promise<HostRecord[]> {
  return invoke('list_hosts');
}

export function saveHost(input: HostInput): Promise<HostRecord> {
  return invoke('save_host', { input });
}

export function deleteHost(id: string): Promise<void> {
  return invoke('delete_host', { id });
}
