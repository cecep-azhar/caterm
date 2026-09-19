// JS <-> Rust binding for vault password policy. Backed by `caterm_core::vault` — see
// crates/caterm-core/src/vault.rs for the source of truth.
import { invoke } from '@tauri-apps/api/core';

export const MIN_VAULT_PASSWORD_LEN = 8;

export function validateVaultPassword(password: string): Promise<void> {
  return invoke('validate_vault_password', { password });
}

export function isVaultInitialized(): Promise<boolean> {
  return invoke('is_vault_initialized');
}

export function resetVault(): Promise<void> {
  return invoke('reset_vault');
}
