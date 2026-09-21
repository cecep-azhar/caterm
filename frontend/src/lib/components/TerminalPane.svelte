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

  let {
    host,
    onSplitRight,
    onSplitDown,
    onClose
  }: {
    host: HostRecord;
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

  let status = $state<'connecting' | 'connected' | 'offline'>('connecting');
  let terminalContainer: HTMLDivElement;

  onMount(() => {
    const term = new Terminal({
      theme: {
        background: '#09090b',
        foreground: '#e4e4e7',
        cursor: '#38bdf8'
      },
      fontFamily: 'Menlo, Monaco, "Courier New", monospace',
      fontSize: 13,
      cursorBlink: true
    });

    const fitAddon = new FitAddon();
    term.loadAddon(fitAddon);
    term.open(terminalContainer);
    fitAddon.fit();

    let disposed = false;
    let session: SshSession | null = null;
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
      term.write(`\r\n\x1b[31m${what}: ${errorMessage(err)}\x1b[0m\r\n`);
    }

    // Injects a snippet command as if it were typed + Enter (T7). Reuses the same
    // echo/sshWrite path as real keystrokes below so the terminal output stays consistent.
    function injectCommand(cmd: string) {
      if (!session) return;
      void sshWrite(session.sessionId, cmd + '\r').catch((err) => reportFailure('Send failed', err));
    }

    function markActive() {
      if (session) {
        setActiveSession({ sessionId: session.sessionId, label: host.label, inject: injectCommand });
      }
    }

    term.writeln(
      `\x1b[1;32mWelcome to CATerm v2\x1b[0m — connecting to \x1b[1;36m${host.label}\x1b[0m (${host.address})...`
    );

    void (async () => {
      try {
        keepUnlisten(
          await onSshOutput(({ sessionId, data }) => {
            if (disposed) return;
            if (session) {
              if (sessionId === session.sessionId) term.write(data);
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
            term.write('\r\n\x1b[33mconnection closed by remote host\x1b[0m\r\n');
          })
        );
      } catch (err) {
        if (!disposed) {
          status = 'offline';
          term.write(
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
        fitAddon.fit();
        void sshResize(opened.sessionId, term.cols, term.rows).catch((err) =>
          reportFailure('Resize failed', err)
        );
        term.write(`\r\n\x1b[32mconnected\x1b[0m (session ${opened.sessionId})\r\n`);

        for (const chunk of early.get(opened.sessionId) ?? []) term.write(chunk);
        early.clear();

        markActive();
        term.focus();
      } catch (err) {
        if (disposed) return;
        status = 'offline';
        term.write(`\r\n\x1b[31mSSH connection failed: ${errorMessage(err)}\x1b[0m\r\n`);
      }
    })();

    term.onData((data) => {
      // Send keystroke to the real Rust SSH PTY backend
      if (session) {
        void sshWrite(session.sessionId, data).catch((err) => reportFailure('Send failed', err));
      }
    });

    const handleResize = () => {
      fitAddon.fit();
      if (session) {
        void sshResize(session.sessionId, term.cols, term.rows).catch(() => {});
      }
    };
    window.addEventListener('resize', handleResize);
    terminalContainer.addEventListener('click', () => {
      markActive();
      term.focus();
    });

    return () => {
      disposed = true;
      for (const unlisten of unlisteners) unlisten();
      unlisteners.length = 0;
      window.removeEventListener('resize', handleResize);
      terminalContainer.removeEventListener('click', markActive);
      if (session) {
        clearActiveSession(session.sessionId);
        void sshDisconnect(session.sessionId).catch(() => {});
      }
      term.dispose();
    };
  });
</script>

<div class="flex flex-col h-full bg-[#09090b] border border-neutral-800 rounded-md overflow-hidden">
  <div class="h-7 sm:h-8 bg-neutral-900/90 border-b border-neutral-800 px-2.5 flex items-center justify-between text-xs font-mono text-neutral-400 shrink-0">
    <div class="flex items-center gap-2 min-w-0">
      <span
        class="w-2 h-2 rounded-full shrink-0 {status === 'connected'
          ? 'bg-emerald-500'
          : status === 'connecting'
            ? 'bg-amber-500'
            : 'bg-neutral-600'}"
      ></span>
      <span class="text-white font-medium truncate">{host.label}</span>
      <span class="text-neutral-500 hidden sm:inline truncate">({host.address})</span>
    </div>
    <div class="flex items-center gap-2 text-neutral-400 shrink-0">
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
