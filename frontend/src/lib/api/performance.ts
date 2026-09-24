import { invoke } from '@tauri-apps/api/core';

/** Stored outside the vault (it is read before unlock, when the window is created). */
export interface PerformancePrefs {
  gpuAcceleration: boolean;
}

export function getPerformancePrefs(): Promise<PerformancePrefs> {
  return invoke('get_performance_prefs');
}

export function setPerformancePrefs(prefs: PerformancePrefs): Promise<void> {
  return invoke('set_performance_prefs', { prefs });
}
