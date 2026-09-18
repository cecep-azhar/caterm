<script lang="ts">
  import { onMount } from 'svelte';
  import { Terminal } from 'xterm';
  import { FitAddon } from '@xterm/addon-fit';
  import 'xterm/css/xterm.css';
  import { sshConnect, sshWrite, sshResize, sshDisconnect, type SshSession } from '$lib/api/ssh';
  import type { HostRecord } from '$lib/api/hosts';
  import { setActiveSession, clearActiveSession } from '$lib/stores/activeSession.svelte';

  let {
    host = null,
    hostLabel = 'Local Terminal',
    hostIp = '127.0.0.1'
  }: { host?: HostRecord | null; hostLabel?: string; hostIp?: string } = $props();

  const label = $derived(host?.label ?? hostLabel);
  const address = $derived(host?.address ?? hostIp);

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
      term.write(cmd);
      void sshWrite(session.sessionId, cmd).catch(() => {});
      term.write('\r\n$ ');
      void sshWrite(session.sessionId, '\r').catch(() => {});
    }

    function markActive() {
      if (session) {
        setActiveSession({ sessionId: session.sessionId, label, inject: injectCommand });
      }
    }

    term.writeln(
      `\x1b[1;32mWelcome to CATerm v2\x1b[0m — connecting to \x1b[1;36m${label}\x1b[0m (${address})...`
    );

    sshConnect({
      hostId: host?.id ?? address,
      address,
      port: host?.port ?? 22,
      username: host?.username ?? 'root'
    })
      .then((opened) => {
        if (disposed) return;
        session = opened;
        status = 'connected';
        term.write(`\r\n\x1b[32mconnected\x1b[0m (session ${opened.sessionId})\r\n$ `);
        markActive();
      })
      .catch((err) => {
        if (disposed) return;
        status = 'offline';
        term.write(
          `\r\n\x1b[33m[placeholder] SSH backend unavailable, using local echo (${String(err)})\x1b[0m\r\n$ `
        );
      });

    term.onData((data) => {
      // Every keystroke is still forwarded to the Rust SSH command even
      // though it's a placeholder today — this is the real Fase 2 wiring
      // point, only the transport underneath is fake for now.
      if (session) {
        void sshWrite(session.sessionId, data).catch(() => {});
      }

      if (data === '\r') {
        term.write('\r\n$ ');
      } else if (data === '') {
        term.write('\b \b');
      } else {
        term.write(data);
      }
    });

    const handleResize = () => {
      fitAddon.fit();
      if (session) {
        void sshResize(session.sessionId, term.cols, term.rows).catch(() => {});
      }
    };
    window.addEventListener('resize', handleResize);
    terminalContainer.addEventListener('click', markActive);

    return () => {
      disposed = true;
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

<div class="flex flex-col h-full bg-[#09090b] border border-neutral-800 rounded-lg overflow-hidden">
  <div class="h-8 bg-neutral-900 border-b border-neutral-800 px-3 flex items-center justify-between text-xs font-mono text-neutral-400">
    <div class="flex items-center gap-2">
      <span
        class="w-2 h-2 rounded-full {status === 'connected'
          ? 'bg-emerald-500'
          : status === 'connecting'
            ? 'bg-amber-500'
            : 'bg-neutral-600'}"
      ></span>
      <span class="text-white font-medium">{label}</span>
      <span class="text-neutral-500">({address})</span>
    </div>
    <div class="flex gap-2 text-neutral-500">
      <span>SSH</span>
      <span>UTF-8</span>
    </div>
  </div>
  <div bind:this={terminalContainer} class="flex-1 p-2 overflow-hidden"></div>
</div>
