<script lang="ts">
  import { onMount } from 'svelte';
  import { Terminal } from 'xterm';
  import { FitAddon } from '@xterm/addon-fit';
  import 'xterm/css/xterm.css';
  import { sshConnect, sshWrite, sshResize, sshDisconnect, sshRead, type SshSession } from '$lib/api/ssh';
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

    // Injects a snippet command as if it were typed + Enter (T7). Reuses the same
    // echo/sshWrite path as real keystrokes below so the terminal output stays consistent.
    function injectCommand(cmd: string) {
      if (!session) return;
      void sshWrite(session.sessionId, cmd + '\r').catch(() => {});
    }

    function markActive() {
      if (session) {
        setActiveSession({ sessionId: session.sessionId, label: host.label, inject: injectCommand });
      }
    }

    term.writeln(
      `\x1b[1;32mWelcome to CATerm v2\x1b[0m — connecting to \x1b[1;36m${host.label}\x1b[0m (${host.address})...`
    );

    let pollTimer: ReturnType<typeof setInterval>;

    sshConnect(host.id)
      .then((opened) => {
        if (disposed) return;
        session = opened;
        status = 'connected';
        fitAddon.fit();
        void sshResize(opened.sessionId, term.cols, term.rows).catch(() => {});
        term.write(`\r\n\x1b[32mconnected\x1b[0m (session ${opened.sessionId})\r\n`);
        markActive();
        term.focus();

        // Start polling for PTY output
        pollTimer = setInterval(() => {
          if (!session) return;
          sshRead(session.sessionId)
            .then((output) => {
              if (output && output.length > 0) {
                term.write(output);
              }
            })
            .catch(() => {});
        }, 40);
      })
      .catch((err) => {
        if (disposed) return;
        status = 'offline';
        term.write(`\r\n\x1b[31mSSH connection failed: ${errorMessage(err)}\x1b[0m\r\n`);
      });

    term.onData((data) => {
      // Send keystroke to the real Rust SSH PTY backend
      if (session) {
        void sshWrite(session.sessionId, data).catch(() => {});
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
      if (pollTimer) clearInterval(pollTimer);
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
        <button onclick={onSplitRight} class="hover:text-sky-400 p-0.5 rounded transition-colors" title="Split Right (Vertical)">
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-width="2" d="M12 3v18M5 3h14a2 2 0 012 2v14a2 2 0 01-2 2H5a2 2 0 01-2-2V5a2 2 0 012-2z"></path></svg>
        </button>
      {/if}
      {#if onSplitDown}
        <button onclick={onSplitDown} class="hover:text-sky-400 p-0.5 rounded transition-colors" title="Split Down (Horizontal)">
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-width="2" d="M3 12h18M5 3h14a2 2 0 012 2v14a2 2 0 01-2 2H5a2 2 0 01-2-2V5a2 2 0 012-2z"></path></svg>
        </button>
      {/if}
      {#if onClose}
        <button onclick={onClose} class="hover:text-rose-400 p-0.5 rounded transition-colors" title="Close Pane">
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-width="2" stroke-linecap="round" d="M6 18L18 6M6 6l12 12"></path></svg>
        </button>
      {/if}
      <span class="ml-1 text-neutral-500 hidden sm:inline">SSH</span>
      <span class="text-neutral-500 hidden sm:inline">UTF-8</span>
    </div>
  </div>
  <div bind:this={terminalContainer} class="flex-1 p-1 overflow-hidden outline-none"></div>
</div>
