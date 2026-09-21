<script lang="ts">
  // Conversational AI assistant. Unlike Prompt Studio's one-shot form, this one is expected to
  // ask before it acts: the backend prompt keeps `ready` false and `steps` empty until the
  // assistant has clarified versions, distro, sudo and anything destructive, so the commands
  // only appear once you have actually agreed to them.
  import { onMount } from 'svelte';
  import {
    aiChat,
    aiExecuteStep,
    type AiChatMessage,
    type AiPlanStep
  } from '$lib/api/ai';
  import { listHosts, type HostRecord } from '$lib/api/hosts';
  import { getActiveSession, injectIntoActiveSession } from '$lib/stores/activeSession.svelte';
  import { showToast } from '$lib/stores/uiNotifications.svelte';

  let { onClose }: { onClose: () => void } = $props();

  type ExecMode = 'ssh' | 'terminal';

  interface StepRun {
    status: 'pending' | 'running' | 'ok' | 'failed';
    output: string;
    exitCode?: number;
  }

  let messages = $state<AiChatMessage[]>([]);
  let draft = $state('');
  let isSending = $state(false);
  let errorMsg = $state('');

  let hosts = $state<HostRecord[]>([]);
  let targetHostId = $state('');
  let execMode = $state<ExecMode>('ssh');

  let proposedSteps = $state<AiPlanStep[]>([]);
  let acceptedSteps = $state<boolean[]>([]);
  let runs = $state<StepRun[]>([]);
  let isExecuting = $state(false);

  let scroller: HTMLDivElement | undefined = $state();

  const targetHost = $derived(hosts.find((h) => h.id === targetHostId));
  const activeSession = $derived(getActiveSession());

  onMount(async () => {
    try {
      hosts = await listHosts();
      if (!targetHostId && hosts.length > 0) targetHostId = hosts[0].id;
    } catch {
      hosts = [];
    }
  });

  function scrollToBottom() {
    requestAnimationFrame(() => {
      if (scroller) scroller.scrollTop = scroller.scrollHeight;
    });
  }

  async function send() {
    const text = draft.trim();
    if (!text || isSending) return;

    errorMsg = '';
    // Steps from an earlier turn no longer describe what is being discussed.
    proposedSteps = [];
    acceptedSteps = [];
    runs = [];

    messages = [...messages, { role: 'user', content: text }];
    draft = '';
    isSending = true;
    scrollToBottom();

    try {
      const reply = await aiChat(messages, targetHost?.label);
      messages = [...messages, { role: 'assistant', content: reply.reply }];
      if (reply.ready && reply.steps.length > 0) {
        proposedSteps = reply.steps;
        acceptedSteps = reply.steps.map(() => true);
        runs = reply.steps.map(() => ({ status: 'pending', output: '' }));
      }
    } catch (err) {
      errorMsg = err instanceof Error ? err.message : String(err);
    } finally {
      isSending = false;
      scrollToBottom();
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    // Enter sends, Shift+Enter makes a new line — the usual chat convention.
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault();
      void send();
    }
  }

  function resetConversation() {
    messages = [];
    proposedSteps = [];
    acceptedSteps = [];
    runs = [];
    errorMsg = '';
  }

  async function runPlan() {
    if (isExecuting || proposedSteps.length === 0) return;

    if (execMode === 'ssh' && !targetHostId) {
      showToast('Pilih host tujuan terlebih dahulu.', 'error');
      return;
    }
    if (execMode === 'terminal' && !activeSession) {
      showToast('Tidak ada terminal aktif — buka sesi dulu atau pakai mode SSH exec.', 'error');
      return;
    }

    isExecuting = true;
    try {
      for (let i = 0; i < proposedSteps.length; i++) {
        if (!acceptedSteps[i]) continue;
        const step = proposedSteps[i];

        if (execMode === 'terminal') {
          // Fire-and-forget: the shell shows the result, we cannot read it back.
          injectIntoActiveSession(step.command);
          runs[i] = { status: 'ok', output: 'Dikirim ke terminal aktif.' };
          continue;
        }

        runs[i] = { status: 'running', output: '' };
        try {
          const result = await aiExecuteStep(targetHostId, step);
          const output = [result.stdout, result.stderr].filter(Boolean).join('\n').trim();
          runs[i] = {
            status: result.success ? 'ok' : 'failed',
            output: output || '(tidak ada output)',
            exitCode: result.exit_code
          };
          if (!result.success) {
            showToast(`Langkah ${i + 1} gagal (exit ${result.exit_code ?? '?'}). Eksekusi dihentikan.`, 'error');
            break;
          }
        } catch (err) {
          runs[i] = {
            status: 'failed',
            output: err instanceof Error ? err.message : String(err)
          };
          break;
        }
      }
    } finally {
      isExecuting = false;
      scrollToBottom();
    }
  }

  const acceptedCount = $derived(acceptedSteps.filter(Boolean).length);
</script>

<aside
  class="fixed right-0 top-0 z-50 h-full w-full sm:w-[26rem] bg-white dark:bg-neutral-950 border-l border-neutral-200 dark:border-neutral-800 shadow-2xl flex flex-col"
  aria-label="AI Assistant"
>
  <header class="p-3 border-b border-neutral-200 dark:border-neutral-800 flex items-center justify-between gap-2 shrink-0">
    <div class="flex items-center gap-2 min-w-0">
      <div class="w-7 h-7 rounded-lg bg-violet-500/15 flex items-center justify-center text-violet-600 dark:text-violet-400 shrink-0">
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
        </svg>
      </div>
      <div class="min-w-0">
        <h2 class="text-sm font-bold text-neutral-900 dark:text-white truncate">AI Assistant</h2>
        <p class="text-[11px] text-neutral-500 truncate">Diskusi dulu, baru dikerjakan</p>
      </div>
    </div>
    <div class="flex items-center gap-1 shrink-0">
      {#if messages.length > 0}
        <button
          onclick={resetConversation}
          class="p-1.5 rounded-lg text-neutral-400 hover:text-neutral-700 dark:hover:text-neutral-200 hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors"
          title="Mulai percakapan baru"
          aria-label="Mulai percakapan baru"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
        </button>
      {/if}
      <button
        onclick={onClose}
        class="p-1.5 rounded-lg text-neutral-400 hover:text-neutral-700 dark:hover:text-neutral-200 hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors"
        aria-label="Tutup AI Assistant"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    </div>
  </header>

  <!-- Target + execution mode -->
  <div class="p-3 border-b border-neutral-200 dark:border-neutral-800 space-y-2 shrink-0">
    <div class="flex items-center gap-2">
      <label for="ai-chat-host" class="text-[11px] font-semibold text-neutral-500 shrink-0">Host</label>
      <select
        id="ai-chat-host"
        bind:value={targetHostId}
        class="flex-1 min-w-0 bg-neutral-50 dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-lg px-2 py-1 text-xs text-neutral-900 dark:text-white focus:outline-none focus:border-violet-500"
      >
        {#if hosts.length === 0}
          <option value="">Belum ada host tersimpan</option>
        {/if}
        {#each hosts as host (host.id)}
          <option value={host.id}>{host.label} ({host.username}@{host.address})</option>
        {/each}
      </select>
    </div>

    <div class="flex items-center gap-2">
      <span class="text-[11px] font-semibold text-neutral-500 shrink-0">Jalankan via</span>
      <div class="flex items-center gap-0.5 p-0.5 rounded-lg border border-neutral-200 dark:border-neutral-800">
        <button
          onclick={() => (execMode = 'ssh')}
          class="px-2 py-0.5 rounded text-[11px] font-medium transition-colors {execMode === 'ssh' ? 'bg-violet-600 text-white' : 'text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white'}"
          title="Channel SSH non-interaktif: output dan exit code ditangkap, semua tercatat di audit log"
        >
          SSH exec
        </button>
        <button
          onclick={() => (execMode = 'terminal')}
          class="px-2 py-0.5 rounded text-[11px] font-medium transition-colors {execMode === 'terminal' ? 'bg-violet-600 text-white' : 'text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white'}"
          title="Ketik ke terminal yang sedang aktif — terlihat langsung, tapi hasilnya tidak bisa dibaca AI"
        >
          Terminal aktif
        </button>
      </div>
    </div>

    {#if execMode === 'terminal'}
      <p class="text-[11px] {activeSession ? 'text-neutral-500' : 'text-amber-600 dark:text-amber-400'}">
        {activeSession
          ? `Akan diketik ke terminal: ${activeSession.label}`
          : 'Belum ada terminal aktif. Buka sesi SSH dulu.'}
      </p>
    {/if}
  </div>

  <!-- Conversation -->
  <div bind:this={scroller} class="flex-1 overflow-y-auto p-3 space-y-3 min-h-0">
    {#if messages.length === 0}
      <div class="text-center py-8 space-y-3">
        <p class="text-sm text-neutral-500 dark:text-neutral-400">
          Ceritakan apa yang ingin dikerjakan di server.
        </p>
        <div class="flex flex-wrap gap-1.5 justify-center">
          {#each ['Bantu setup Docker', 'Install Node.js untuk produksi', 'Hardening SSH & firewall'] as suggestion}
            <button
              onclick={() => {
                draft = suggestion;
                void send();
              }}
              class="px-2.5 py-1 rounded-full text-[11px] border border-neutral-200 dark:border-neutral-800 text-neutral-600 dark:text-neutral-300 hover:border-violet-400 hover:text-violet-600 dark:hover:text-violet-400 transition-colors"
            >
              {suggestion}
            </button>
          {/each}
        </div>
        <p class="text-[11px] text-neutral-400 dark:text-neutral-500 max-w-xs mx-auto">
          AI akan bertanya dulu (versi, distro, sudo) sebelum mengusulkan perintah apa pun.
        </p>
      </div>
    {/if}

    {#each messages as message, i (i)}
      <div class="flex {message.role === 'user' ? 'justify-end' : 'justify-start'}">
        <div
          class="max-w-[85%] rounded-xl px-3 py-2 text-xs leading-relaxed whitespace-pre-wrap break-words {message.role === 'user'
            ? 'bg-violet-600 text-white'
            : 'bg-neutral-100 dark:bg-neutral-900 text-neutral-800 dark:text-neutral-200 border border-neutral-200 dark:border-neutral-800'}"
        >
          {message.content}
        </div>
      </div>
    {/each}

    {#if isSending}
      <div class="flex justify-start">
        <div class="rounded-xl px-3 py-2 text-xs bg-neutral-100 dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 text-neutral-500">
          AI sedang berpikir...
        </div>
      </div>
    {/if}

    {#if errorMsg}
      <div class="rounded-lg px-3 py-2 text-xs bg-rose-50 dark:bg-rose-950/40 border border-rose-200 dark:border-rose-900 text-rose-700 dark:text-rose-300">
        <p class="font-semibold mb-0.5">Gagal menghubungi AI</p>
        <p class="break-words">{errorMsg}</p>
        <a href="/settings" class="underline mt-1 inline-block">Periksa Settings &gt; AI Assistant</a>
      </div>
    {/if}

    <!-- Proposal: only present once the assistant says it is ready -->
    {#if proposedSteps.length > 0}
      <div class="rounded-xl border border-violet-300 dark:border-violet-900 bg-violet-50 dark:bg-violet-950/30 p-3 space-y-2">
        <div class="flex items-center justify-between gap-2">
          <h3 class="text-xs font-bold text-violet-800 dark:text-violet-300">
            Rencana eksekusi ({proposedSteps.length} langkah)
          </h3>
          <span class="text-[11px] text-violet-700 dark:text-violet-400">{acceptedCount} dipilih</span>
        </div>

        <ul class="space-y-1.5">
          {#each proposedSteps as step, i (i)}
            <li class="rounded-lg bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 p-2">
              <label class="flex items-start gap-2 cursor-pointer">
                <input
                  type="checkbox"
                  bind:checked={acceptedSteps[i]}
                  disabled={isExecuting}
                  class="mt-0.5 accent-violet-600 shrink-0"
                />
                <span class="min-w-0 flex-1">
                  <span class="flex items-center gap-1.5 flex-wrap">
                    <span class="text-[11px] font-semibold text-neutral-800 dark:text-neutral-200">
                      {i + 1}. {step.title}
                    </span>
                    {#if step.is_danger || step.is_sudo}
                      <span class="px-1 py-0.5 rounded text-[9px] font-bold uppercase bg-amber-500/20 text-amber-700 dark:text-amber-400">
                        {step.is_danger ? 'Berisiko' : 'Sudo'}
                      </span>
                    {/if}
                  </span>
                  <code class="block mt-1 text-[11px] font-mono text-sky-700 dark:text-sky-400 break-all">
                    {step.command}
                  </code>
                  {#if step.description}
                    <span class="block mt-0.5 text-[11px] text-neutral-500">{step.description}</span>
                  {/if}
                </span>
              </label>

              {#if runs[i] && runs[i].status !== 'pending'}
                <div class="mt-1.5 pl-6">
                  <span
                    class="text-[10px] font-semibold uppercase {runs[i].status === 'ok'
                      ? 'text-emerald-600 dark:text-emerald-400'
                      : runs[i].status === 'failed'
                        ? 'text-rose-600 dark:text-rose-400'
                        : 'text-amber-600 dark:text-amber-400'}"
                  >
                    {runs[i].status === 'running' ? 'Berjalan...' : runs[i].status}
                    {#if runs[i].exitCode !== undefined}· exit {runs[i].exitCode}{/if}
                  </span>
                  {#if runs[i].output}
                    <pre class="mt-1 p-1.5 rounded bg-neutral-950 text-neutral-200 text-[10px] font-mono whitespace-pre-wrap break-all max-h-32 overflow-y-auto">{runs[i].output}</pre>
                  {/if}
                </div>
              {/if}
            </li>
          {/each}
        </ul>

        <button
          onclick={runPlan}
          disabled={isExecuting || acceptedCount === 0}
          class="w-full px-3 py-2 rounded-lg text-xs font-semibold bg-violet-600 hover:bg-violet-500 disabled:opacity-50 text-white transition-colors"
        >
          {isExecuting
            ? 'Menjalankan...'
            : execMode === 'ssh'
              ? `Jalankan ${acceptedCount} langkah di ${targetHost?.label ?? 'host'}`
              : `Kirim ${acceptedCount} langkah ke terminal aktif`}
        </button>
      </div>
    {/if}
  </div>

  <!-- Composer -->
  <div class="p-3 border-t border-neutral-200 dark:border-neutral-800 shrink-0">
    <div class="flex items-end gap-2">
      <textarea
        bind:value={draft}
        onkeydown={handleKeydown}
        rows="2"
        placeholder="Misal: bantu setup Docker di server ini"
        aria-label="Pesan untuk AI"
        class="flex-1 resize-none bg-neutral-50 dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-lg px-2.5 py-2 text-xs text-neutral-900 dark:text-white placeholder-neutral-400 focus:outline-none focus:border-violet-500"
      ></textarea>
      <button
        onclick={send}
        disabled={isSending || draft.trim().length === 0}
        class="px-3 py-2 rounded-lg bg-violet-600 hover:bg-violet-500 disabled:opacity-40 text-white transition-colors shrink-0"
        aria-label="Kirim"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8" />
        </svg>
      </button>
    </div>
  </div>
</aside>
