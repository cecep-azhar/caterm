// Thin wrapper over @tauri-apps/plugin-updater. The check and the signature verification both
// run in Rust against `plugins.updater` in tauri.conf.json (GitHub `latest.json` + minisign
// pubkey); nothing here talks to the network directly.
import { check, type Update, type DownloadEvent } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

export type { Update, DownloadEvent };

export function isTauriRuntime(): boolean {
  return typeof window !== 'undefined' && Boolean((window as any).__TAURI_INTERNALS__);
}

export interface UpdateProgress {
  downloaded: number;
  total: number | null;
  percent: number | null;
  phase: 'idle' | 'started' | 'downloading' | 'finished';
}

/** Resolves to the pending update, or null when this build is already the latest. */
export async function checkForAppUpdate(): Promise<Update | null> {
  if (!isTauriRuntime()) {
    throw new Error('Updates are only available in the desktop app.');
  }
  return check();
}

/**
 * Downloads, verifies and installs `update`.
 * Progress callback reports bytes downloaded, total bytes, and status phase.
 */
export async function downloadAndInstallUpdate(
  update: Update,
  onProgress?: (progress: UpdateProgress) => void
): Promise<void> {
  let downloaded = 0;
  let total: number | null = null;

  await update.downloadAndInstall((event: DownloadEvent) => {
    if (event.event === 'Started') {
      total = event.data.contentLength ?? null;
      onProgress?.({
        downloaded,
        total,
        percent: total ? Math.min(100, Math.round((downloaded / total) * 100)) : null,
        phase: 'started'
      });
    } else if (event.event === 'Progress') {
      downloaded += event.data.chunkLength;
      onProgress?.({
        downloaded,
        total,
        percent: total ? Math.min(100, Math.round((downloaded / total) * 100)) : null,
        phase: 'downloading'
      });
    } else if (event.event === 'Finished') {
      onProgress?.({
        downloaded: total ?? downloaded,
        total,
        percent: 100,
        phase: 'finished'
      });
    }
  });
}

/**
 * Restart the application to apply the installed update.
 */
export async function relaunchApp(): Promise<void> {
  await relaunch();
}

// Backwards compatibility helper
export async function installAppUpdate(
  update: Update,
  onProgress?: (downloaded: number, total: number | null) => void
): Promise<void> {
  await downloadAndInstallUpdate(update, (p) => {
    onProgress?.(p.downloaded, p.total);
  });
  await relaunchApp();
}
