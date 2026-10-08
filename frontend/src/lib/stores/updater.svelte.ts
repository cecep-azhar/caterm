// Shared update state: modal popup, toast, and Settings > Updates read the same
// check result, so an update found in one place is reflected everywhere.
import {
  checkForAppUpdate,
  downloadAndInstallUpdate,
  relaunchApp,
  isTauriRuntime,
  type Update,
  type UpdateProgress
} from '$lib/api/updater';
import { errorText } from '$lib/errors';
import { showToast } from '$lib/stores/uiNotifications.svelte';
import { t } from '$lib/i18n/index.svelte';

export type UpdateStatus =
  | 'idle'
  | 'checking'
  | 'up-to-date'
  | 'available'
  | 'downloading'
  | 'ready-to-restart'
  | 'error';

let status = $state<UpdateStatus>('idle');
let update = $state.raw<Update | null>(null);
let errorMessage = $state('');
let progress = $state<UpdateProgress>({
  downloaded: 0,
  total: null,
  percent: null,
  phase: 'idle'
});
let toastDismissed = $state(false);
let isModalOpen = $state(false);
let periodicTimer: ReturnType<typeof setInterval> | null = null;

export function getUpdater() {
  return {
    get status() {
      return status;
    },
    get version() {
      return update?.version ?? '';
    },
    get currentVersion() {
      return update?.currentVersion ?? '';
    },
    get body() {
      return update?.body ?? '';
    },
    get date() {
      return update?.date ?? '';
    },
    get error() {
      return errorMessage;
    },
    get progress() {
      return progress;
    },
    get isModalOpen() {
      return isModalOpen;
    },
    set isModalOpen(open: boolean) {
      isModalOpen = open;
    },
    /** The floating toast shows only when update is available/downloading and modal isn't open */
    get showToast() {
      return (
        (status === 'available' || status === 'downloading' || status === 'ready-to-restart') &&
        !toastDismissed &&
        !isModalOpen
      );
    }
  };
}

/**
 * Checks for available updates.
 * `silent` is used for background checks: failures stay silent.
 * `showModalOnFound` opens the update modal automatically when update is available.
 */
export async function checkForUpdates({
  silent = false,
  showModalOnFound = false
}: {
  silent?: boolean;
  showModalOnFound?: boolean;
} = {}): Promise<void> {
  if (status === 'checking' || status === 'downloading') return;
  if (silent && !isTauriRuntime()) return;

  status = 'checking';
  errorMessage = '';

  try {
    update = await checkForAppUpdate();
    if (update) {
      status = 'available';
      toastDismissed = false;
      if (showModalOnFound) {
        isModalOpen = true;
      }
    } else {
      status = 'up-to-date';
    }
  } catch (err) {
    update = null;
    if (silent) {
      console.warn('Background update check failed:', err);
      status = 'idle';
    } else {
      errorMessage = errorText(err);
      status = 'error';
    }
  }
}

/**
 * Downloads update with interactive progress callback.
 */
export async function startDownloadUpdate(): Promise<void> {
  if (!update || status === 'downloading') return;

  status = 'downloading';
  errorMessage = '';
  progress = {
    downloaded: 0,
    total: null,
    percent: 0,
    phase: 'started'
  };

  try {
    await downloadAndInstallUpdate(update, (p) => {
      progress = p;
    });
    status = 'ready-to-restart';
    progress = {
      ...progress,
      percent: 100,
      phase: 'finished'
    };
  } catch (err) {
    errorMessage = errorText(err);
    status = 'error';
    showToast(t('updater.failed', { error: errorMessage }), 'error');
  }
}

/**
 * Restarts the app to apply the installed update.
 */
export async function restartAndApplyUpdate(): Promise<void> {
  try {
    await relaunchApp();
  } catch (err) {
    errorMessage = errorText(err);
    showToast(t('updater.failed', { error: errorMessage }), 'error');
  }
}

/**
 * Compatibility wrapper for existing callers.
 */
export async function installUpdate(): Promise<void> {
  if (status === 'ready-to-restart') {
    await restartAndApplyUpdate();
    return;
  }
  await startDownloadUpdate();
}

export function openUpdateModal() {
  isModalOpen = true;
}

export function closeUpdateModal() {
  isModalOpen = false;
}

export function dismissUpdateToast() {
  toastDismissed = true;
}

/**
 * Starts periodic background update checks (default every 4 hours).
 */
export function initPeriodicUpdateChecker(intervalMs = 4 * 60 * 60 * 1000) {
  if (periodicTimer) return;
  if (!isTauriRuntime()) return;

  // Immediate quiet check at startup
  void checkForUpdates({ silent: true, showModalOnFound: true });

  periodicTimer = setInterval(() => {
    void checkForUpdates({ silent: true, showModalOnFound: true });
  }, intervalMs);
}

export function stopPeriodicUpdateChecker() {
  if (periodicTimer) {
    clearInterval(periodicTimer);
    periodicTimer = null;
  }
}
