// Appearance/performance prefs kept in localStorage. Separate from profile.svelte.ts
// because these are device-level preferences, not identity.

const STORAGE_KEY = 'caterm_appearance_v1';

interface StoredAppearance {
  reduceMotion: boolean;
}

function load(): StoredAppearance | null {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as Partial<StoredAppearance>;
    if (typeof parsed.reduceMotion !== 'boolean') return null;
    return { reduceMotion: parsed.reduceMotion };
  } catch {
    return null;
  }
}

const initial = typeof localStorage === 'undefined' ? null : load();
// Decorative canvas backgrounds (lock screen grid flow) are GPU-accelerated and
// keep a WebView2 GPU process resident for the whole session, so they default to
// off; users who want the visual flair back can opt in from Settings.
let reduceMotion = $state(initial?.reduceMotion ?? true);

export function getAppearance() {
  return {
    get reduceMotion() {
      return reduceMotion;
    }
  };
}

export function setReduceMotion(next: boolean) {
  reduceMotion = next;
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify({ reduceMotion }));
  } catch {
    // Storage blocked: the change still applies for this session.
  }
}
