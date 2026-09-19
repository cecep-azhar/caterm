// Tracks which host sessions are actually open right now. This is the single source of
// truth for "is there a session" — the Session page opens tabs into it (from a host picked
// on the Hosts/Groups page, via the `?host=`/`?hosts=` query params) and the top nav renders
// exactly what's here, nothing hardcoded. Module-level `$state` survives client-side
// navigation between routes (SPA), so switching to Dashboard and back to Session keeps tabs.

import type { HostRecord } from '$lib/api/hosts';

export interface SessionTab {
  /** One tab per host — using the host id keeps re-opening the same host a no-op. */
  id: string;
  host: HostRecord;
}

let tabs = $state<SessionTab[]>([]);

export function getTabs(): SessionTab[] {
  return tabs;
}

export function openTab(host: HostRecord) {
  if (!tabs.some((t) => t.id === host.id)) {
    tabs = [...tabs, { id: host.id, host }];
  }
}

export function closeTab(id: string) {
  tabs = tabs.filter((t) => t.id !== id);
}

export function closeAllTabs() {
  tabs = [];
}
