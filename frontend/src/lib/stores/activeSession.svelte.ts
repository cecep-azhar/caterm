// Tracks which connected TerminalPane is "active" so Snippets can inject a command into it
// (T7). A pane registers itself active on connect and whenever its terminal gets focus/click,
// and clears itself on unmount. This is intentionally page-agnostic: Session and any other
// route hosting a TerminalPane share the same active-session slot.

export interface ActiveSession {
  sessionId: string;
  tabId?: string;
  label: string;
  inject: (command: string) => void;
  write?: (data: string) => void;
}

let current = $state<ActiveSession | null>(null);
const allSessions = new Map<string, ActiveSession>();

export function getActiveSession(): ActiveSession | null {
  return current;
}

export function getAllActiveSessions(): ActiveSession[] {
  return Array.from(allSessions.values());
}

export function setActiveSession(session: ActiveSession) {
  current = session;
  allSessions.set(session.sessionId, session);
}

export function registerSessionPane(session: ActiveSession) {
  allSessions.set(session.sessionId, session);
}

export function unregisterSessionPane(sessionId: string) {
  allSessions.delete(sessionId);
  if (current?.sessionId === sessionId) {
    current = null;
  }
}

export function clearActiveSession(sessionId: string) {
  unregisterSessionPane(sessionId);
}

export function hasActiveSession(): boolean {
  return current !== null;
}

/** Injects `command` into the active terminal session. Returns false if none is connected. */
export function injectIntoActiveSession(command: string): boolean {
  if (!current) return false;
  current.inject(command);
  return true;
}

/** Injects `command` into ALL registered terminal sessions. */
export function injectIntoAllSessions(command: string): number {
  let count = 0;
  for (const session of allSessions.values()) {
    session.inject(command);
    count++;
  }
  return count;
}

/** Broadcasts raw input data across all registered terminal sessions except optionally the originator. */
export function broadcastWriteToAll(data: string, exceptSessionId?: string): number {
  let count = 0;
  for (const session of allSessions.values()) {
    if (exceptSessionId && session.sessionId === exceptSessionId) continue;
    if (session.write) {
      session.write(data);
      count++;
    }
  }
  return count;
}

