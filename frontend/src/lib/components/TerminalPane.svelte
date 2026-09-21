<script lang="ts">
  import { onMount } from 'svelte';
  import { Terminal } from 'xterm';
  import { FitAddon } from '@xterm/addon-fit';
  import 'xterm/css/xterm.css';
  import type { UnlistenFn } from '@tauri-apps/api/event';
  import {
    sshConnect,
    sshWrite,
    sshResize,
    sshDisconnect,
    onSshOutput,
    onSshClosed,
    type SshSession
  } from '$lib/api/ssh';
  import type { HostRecord } from '$lib/api/hosts';
  import { setActiveSession, clearActiveSession } from '$lib/stores/activeSession.svelte';
  import { getTheme, terminalTheme } from '$lib/stores/theme.svelte';

  let {
    host,
    label,
    isActive = true,
    onSplitRight,
    onSplitDown,
    onClose
  }: {
    host: HostRecord;
    /** Disambiguated tab name, e.g. "YPC (2)" when the host is open more than once. */
    label?: string;
    /** Whether this pane is the one the user is looking at (drives focus + snippet target). */
    isActive?: boolean;
    onSplitRight?: () => void;
    onSplitDown?: () => void;
    onClose?: () => void;
  } = $props();

  function errorMessage(err: unknown): string {
    if (err && typeof err === 'object' && 'message' in err) {
      return String((err as { message?: unknown }).message);
    }
    return String(err);
  }

  const paneLabel = $derived(label ?? host.label);
  const theme = getTheme();
  let status = $state<'connecting' | 'connected' | 'offline'>('connecting');
  let terminalContainer: HTMLDivElement;

  // Component scope, not `onMount` scope: a pane can become the visible one long after it
  // mounted, and the effect below needs to reach the live session to retarget snippets and
  // re-fit the viewport.
  let session = $state<SshSession | null>(null);
  let term: Terminal | null = null;
  let fitAddon: FitAddon | null = null;

  // Injects a snippet command as if it were typed + Enter (T7). Reuses the same
  // sshWrite path as real keystrokes so the terminal output stays consistent.
  function injectCommand(cmd: string) {
    if (!session) return;
    void sshWrite(session.sessionId, cmd + '\r').catch(() => {});
  }

  function markActive() {
    if (session) {
      setActiveSession({ sessionId: session.sessionId, label: paneLabel, inject: injectCommand });
    }
  }

  // xterm owns its own canvas and never sees Tailwind's `dark:` classes, so the palette has to
  // be pushed in whenever the theme flips — otherwise the terminal stays black in light mode.
  $effect(() => {
    const palette = terminalTheme(theme.name);
    if (term) term.options.theme = palette;
  });

  // Panes are kept mounted while hidden so their SSH session and scrollback survive tab
  // switching. A hidden pane keeps its layout box (visibility, not display), so xterm stays
  // correctly sized — the re-fit here is belt and braces for a resize that happened while away.
  $effect(() => {
    if (!isActive || !session) return;
    markActive();
    fitAddon?.fit();
    if (term) {
      void sshResize(session.sessionId, term.cols, term.rows).catch(() => {});
      term.focus();
    }
  });

  onMount(() => {
    const terminal = new Terminal({
      theme: terminalTheme(theme.name),
      fontFamily: 'Menlo, Monaco, "Courier New", monospace',
      fontSize: 13,
      cursorBlink: true
    });
    term = terminal;

    const fit = new FitAddon();
    fitAddon = fit;
    terminal.loadAddon(fit);
    terminal.open(terminalContainer);
    fit.fit();

    let disposed = false;
    const unlisteners: UnlistenFn[] = [];

    // Output that arrived while `sshConnect` was still in flight — we subscribe *before*
    // connecting (so the shell banner can't be missed) but only learn our own session id
    // afterwards, so anything that lands in between is parked here and flushed on arrival.
    const early = new Map<string, string[]>();

    function keepUnlisten(unlisten: UnlistenFn) {
      if (disposed) unlisten();
      else unlisteners.push(unlisten);
    }

    // A backend call that fails silently is how the blank-terminal bug stayed invisible for a
    // whole release: `ssh_read` was rejected by the Tauri ACL on every poll and every rejection
    // was swallowed by an empty `.catch()`. Failures now show up in the terminal itself, once
    // per pane so a flapping link can't spam the scrollback.
    let reportedFailure = false;
    function reportFailure(what: string, err: unknown) {
      if (disposed || reportedFailure) return;
      reportedFailure = true;
      terminal.write(`\r\n\x1b[31m${what}: ${errorMessage(err)}\x1b[0m\r\n`);
    }

    terminal.writeln(
      `\x1b[1;32mWelcome to CATerm v2\x1b[0m — connecting to \x1b[1;36m${host.label}\x1b[0m (${host.address})...`
    );

    void (async () => {
      try {
        keepUnlisten(
          await onSshOutput(({ sessionId, data }) => {
            if (disposed) return;
            if (session) {
              if (sessionId === session.sessionId) terminal.write(data);
              return;
            }
            const buffered = early.get(sessionId) ?? [];
            buffered.push(data);
            early.set(sessionId, buffered);
          })
        );

        keepUnlisten(
          await onSshClosed(({ sessionId }) => {
            if (disposed || !session || sessionId !== session.sessionId) return;
            status = 'offline';
            terminal.write('\r\n\x1b[33mconnection closed by remote host\x1b[0m\r\n');
          })
        );
      } catch (err) {
        if (!disposed) {
          status = 'offline';
          terminal.write(
            `\r\n\x1b[31mTidak bisa berlangganan output terminal: ${errorMessage(err)}\x1b[0m\r\n`
          );
        }
        return;
      }

      try {
        const opened = await sshConnect(host.id);
        if (disposed) {
          void sshDisconnect(opened.sessionId).catch(() => {});
          return;
        }

        session = opened;
        status = 'connected';
        fit.fit();
        void sshResize(opened.sessionId, terminal.cols, terminal.rows).catch((err) =>
          reportFailure('Resize failed', err)
        );
        terminal.write(`\r\n\x1b[32mconnected\x1b[0m (session ${opened.sessionId})\r\n`);

        for (const chunk of early.get(opened.sessionId) ?? []) terminal.write(chunk);
        early.clear();
      } catch (err) {
        if (disposed) return;
        status = 'offline';
        terminal.write(`\r\n\x1b[31mSSH connection failed: ${errorMessage(err)}\x1b[0m\r\n`);
      }
    })();

    terminal.onData((data) => {
      // Send keystroke to the real Rust SSH PTY backend
      if (session) {
        void sshWrite(session.sessionId, data).catch((err) => reportFailure('Send failed', err));
      }
    });

    // Watch the container, not the window. A pane is resized by things the window never sees:
    // switching split layout, toggling the SFTP panel, collapsing the sidebar. Panes now stay
    // mounted across all of those (so sessions survive), which means nothing else would ever
    // re-measure xterm and the terminal would keep rendering at its old column count.
    let fitQueued = false;
    const applyFit = () => {
      fitQueued = false;
      // A hidden pane has a zero-height box only if something collapses it; re-fitting to 0
      // would destroy the buffer geometry, so skip and wait until it is laid out again.
      if (terminalContainer.clientWidth === 0 || terminalContainer.clientHeight === 0) return;
      fit.fit();
      if (session) {
        void sshResize(session.sessionId, terminal.cols, terminal.rows).catch(() => {});
      }
    };
    const observer = new ResizeObserver(() => {
      if (fitQueued) return;
      fitQueued = true;
      requestAnimationFrame(applyFit);
    });
    observer.observe(terminalContainer);

    const handleClick = () => {
      markActive();
      terminal.focus();
    };
    terminalContainer.addEventListener('click', handleClick);

    return () => {
      disposed = true;
      for (const unlisten of unlisteners) unlisten();
      unlisteners.length = 0;
      observer.disconnect();
      terminalContainer.removeEventListener('click', handleClick);
      if (session) {
        clearActiveSession(session.sessionId);
        void sshDisconnect(session.sessionId).catch(() => {});
      }
      term = null;
      fitAddon = null;
      terminal.dispose();
    };
  });
</script>

<div class="flex flex-col h-full bg-white dark:bg-[#09090b] border border-neutral-200 dark:border-neutral-800 rounded-md overflow-hidden">
  <div class="h-7 sm:h-8 bg-neutral-100 dark:bg-neutral-900/90 border-b border-neutral-200 dark:border-neutral-800 px-2.5 flex items-center justify-between text-xs font-mono text-neutral-500 dark:text-neutral-400 shrink-0">
    <div class="flex items-center gap-2 min-w-0">
      <span
        class="w-2 h-2 rounded-full shrink-0 {status === 'connected'
          ? 'bg-emerald-500'
          : status === 'connecting'
            ? 'bg-amber-500'
            : 'bg-neutral-600'}"
      ></span>
      <span class="text-neutral-900 dark:text-white font-medium truncate">{paneLabel}</span>
      <span class="text-neutral-500 dark:text-neutral-500 hidden sm:inline truncate">({host.address})</span>
    </div>
    <div class="flex items-center gap-2 text-neutral-500 dark:text-neutral-400 shrink-0">
      {#if onSplitRight}
        <button onclick={onSplitRight} class="hover:text-sky-400 p-0.5 rounded transition-colors" title="Split Right (Vertical)" aria-label="Split Right">
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-width="2" d="M12 3v18M5 3h14a2 2 0 012 2v14a2 2 0 01-2 2H5a2 2 0 01-2-2V5a2 2 0 012-2z"></path></svg>
        </button>
      {/if}
      {#if onSplitDown}
        <button onclick={onSplitDown} class="hover:text-sky-400 p-0.5 rounded transition-colors" title="Split Down (Horizontal)" aria-label="Split Down">
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-width="2" d="M3 12h18M5 3h14a2 2 0 012 2v14a2 2 0 01-2 2H5a2 2 0 01-2-2V5a2 2 0 012-2z"></path></svg>
        </button>
      {/if}
      {#if onClose}
        <button onclick={onClose} class="hover:text-rose-400 p-0.5 rounded transition-colors" title="Close Pane" aria-label="Close Pane">
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-width="2" stroke-linecap="round" d="M6 18L18 6M6 6l12 12"></path></svg>
        </button>
      {/if}
      <span class="ml-1 text-neutral-500 hidden sm:inline">SSH</span>
      <span class="text-neutral-500 hidden sm:inline">UTF-8</span>
    </div>
  </div>
  <div bind:this={terminalContainer} class="flex-1 p-1 overflow-hidden outline-none"></div>
</div>
