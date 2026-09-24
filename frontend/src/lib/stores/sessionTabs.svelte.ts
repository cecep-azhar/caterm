// Tracks which host sessions are actually open right now. This is the single source of
// truth for "is there a session" — the Session page opens tabs into it (from a host picked
// on the Hosts/Groups page, via the `?host=`/`?hosts=` query params) and the top nav renders
// exactly what's here, nothing hardcoded. Module-level `$state` survives client-side
// navigation between routes (SPA), so switching to Dashboard and back to Session keeps tabs.
//
// A tab is one SSH *session*, not one host: the same host can be open several times at once,
// each with its own PTY. Tab ids are therefore generated, never the host id — keying tabs by
// host id used to make a second Connect to an already-open host a silent no-op.

import type { HostRecord } from '$lib/api/hosts';
import { recordSessionClosed } from '$lib/stores/feedbackStore.svelte';
import { markHostUsed } from '$lib/stores/hostPrefs.svelte';
import { setShowFiles } from '$lib/stores/sessionView.svelte';

export interface SessionTab {
  /** Unique per open session. Never the host id — a host may have several tabs. */
  id: string;
  host: HostRecord;
  /** 1-based position among the currently open tabs for this same host. */
  seq: number;
  /** Name the user gave this tab (right-click > Rename). Unset = derived from the host. */
  title?: string;
}

let tabs = $state<SessionTab[]>([]);
let counter = 0;

export function getTabs(): SessionTab[] {
  return tabs;
}

/**
 * Smallest positive number not currently taken by another tab on the same host, so closing
 * "YPC (2)" and opening a new one reuses that label instead of drifting to "YPC (5)".
 */
function nextSeq(hostId: string): number {
  const taken = new Set(tabs.filter((t) => t.host.id === hostId).map((t) => t.seq));
  let seq = 1;
  while (taken.has(seq)) seq++;
  return seq;
}

/** Always opens a *new* session. Returns the new tab's id so the caller can focus it. */
export function openTab(host: HostRecord): string {
  counter += 1;
  const tab: SessionTab = {
    id: `tab-${host.id}-${counter}`,
    host,
    seq: nextSeq(host.id)
  };
  tabs = [...tabs, tab];
  markHostUsed(host.id);
  // A new connection lands on a full-width terminal; Files opens only when asked for.
  setShowFiles(false);
  return tab.id;
}

/** Opens a session for `host` only if none is open yet; returns the existing or new tab id. */
export function openTabOnce(host: HostRecord): string {
  const existing = tabs.find((t) => t.host.id === host.id);
  return existing ? existing.id : openTab(host);
}

export function closeTab(id: string) {
  tabs = tabs.filter((t) => t.id !== id);
  recordSessionClosed();
}

/**
 * Blank or whitespace-only resets the tab to its host-derived label. Mutates in place rather
 * than replacing the tab object, so nothing keyed on the object (the live terminal) re-runs.
 */
export function renameTab(id: string, title: string) {
  const tab = tabs.find((t) => t.id === id);
  if (tab) tab.title = title.trim() || undefined;
}

export function closeAllTabs() {
  tabs = [];
}

/** What the tab strip shows: plain label for the first, numbered for additional sessions. */
export function tabLabel(tab: SessionTab): string {
  if (tab.title) return tab.title;
  return tab.seq > 1 ? `${tab.host.label} (${tab.seq})` : tab.host.label;
}
