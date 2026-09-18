// JS <-> Rust binding for vault password policy. Backed by `caterm_core::vault` — see
// crates/caterm-core/src/vault.rs for the source of truth (currently just the minimum
// length rule; full password-derived unlock is later Fase 1 work).
import { invoke } from '@tauri-apps/api/core';

export const MIN_VAULT_PASSWORD_LEN = 8;

export function validateVaultPassword(password: string): Promise<void> {
  return invoke('validate_vault_password', { password });
}
