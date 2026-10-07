<script lang="ts">
  // Conversational AI assistant (Floating Smart Card layout).
  import { onMount } from 'svelte';
  import {
    aiChat,
    aiExecuteStep,
    type AiChatMessage,
    type AiPlanStep
  } from '$lib/api/ai';
  import { listHosts, type HostRecord } from '$lib/api/hosts';
  import { getTabs, localTerminalHost, LOCAL_HOST_ID } from '$lib/stores/sessionTabs.svelte';
  import { getSessionView } from '$lib/stores/sessionView.svelte';
  import { getActiveSession, injectIntoActiveSession } from '$lib/stores/activeSession.svelte';
  import { getProfile, saveProfile } from '$lib/stores/profile.svelte';
  import { saveHost, deleteHost } from '$lib/api/hosts';
  import { saveSnippet, deleteSnippet } from '$lib/api/snippets';
  import { saveGroup, deleteGroup } from '$lib/api/groups';
  import { invoke } from '@tauri-apps/api/core';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import { showToast } from '$lib/stores/uiNotifications.svelte';
  import { errorText } from '$lib/errors';
  import { t } from '$lib/i18n/index.svelte';

  let { onClose }: { onClose: () => void } = $props();

  type ExecMode = 'ssh' | 'terminal';

  interface StepRun {
    status: 'pending' | 'running' | 'ok' | 'failed';
    output: string;
    exitCode?: number;
  }

  let isMinimized = $state(false);
  let isFullHeight = $state(false);
  let showConfirmClear = $state(false);
  let aiDisabledWarning = $state(false);

  const initialGreeting = 'Halo! Saya Hana AI, asisten DevOps & Terminal CATerm. Pilih target host dan tanyakan perintah server atau otomatisasi infrastruktur Anda.';

  let messages = $state<AiChatMessage[]>([
    {
      role: 'assistant',
      content: initialGreeting
    }
  ]);
  let draft = $state('');
  let isSending = $state(false);
  let errorMsg = $state('');

  let hosts = $state<HostRecord[]>([]);
  let targetHostId = $state<string>(LOCAL_HOST_ID);
  let execMode = $state<ExecMode>('ssh');

  let proposedSteps = $state<AiPlanStep[]>([]);
  let acceptedSteps = $state<boolean[]>([]);
  let runs = $state<StepRun[]>([]);
  let isExecuting = $state(false);

  let scroller: HTMLDivElement | undefined = $state();

  const allHosts = $derived([localTerminalHost(), ...hosts]);
  const targetHost = $derived(allHosts.find((h) => h.id === targetHostId) || (targetHostId === LOCAL_HOST_ID ? localTerminalHost() : undefined));
  const activeSession = $derived(getActiveSession());

  onMount(async () => {
    try {
      hosts = await listHosts();
      const activeTabs = getTabs();
      const view = getSessionView();
      const activeTab = activeTabs.find((t) => t.id === view.selectedTabId) || activeTabs[0];
      if (activeTab?.host?.id) {
        targetHostId = activeTab.host.id;
      } else if (hosts.length > 0) {
        targetHostId = LOCAL_HOST_ID;
      }
    } catch {
      hosts = [];
      targetHostId = LOCAL_HOST_ID;
    }
  });

  function scrollToBottom() {
    requestAnimationFrame(() => {
      if (scroller) scroller.scrollTop = scroller.scrollHeight;
    });
  }

  function cleanMessageContent(content: string): string {
    return content.replace(/\n?\[Active UI Context:[^\]]*\]/g, '').trim();
  }

  async function send() {
    const text = draft.trim();
    if (!text || isSending) return;

    errorMsg = '';
    aiDisabledWarning = false;
    proposedSteps = [];
    acceptedSteps = [];
    runs = [];

    const currentRoute = page.url.pathname;
    const currentTab = page.url.searchParams.get('tab') || '';
    const profile = getProfile();
    const uiContext = `[Active UI Context: Route="${currentRoute}", Tab="${currentTab}", CurrentProfileName="${profile.name}", HostCount=${hosts.length}]`;

    messages = [...messages, { role: 'user', content: `${text}\n${uiContext}` }];
    draft = '';
    isSending = true;
    scrollToBottom();

    try {
      const reply = await aiChat(messages, targetHost?.label);
      messages = [...messages, { role: 'assistant', content: reply.reply }];
      
      let stepsToPropose = reply.steps || [];

      // Fallback: If assistant replied with code blocks but didn't output structured steps, extract them!
      if (stepsToPropose.length === 0) {
        const codeBlockRegex = /```(?:bash|sh|shell)?\n([\s\S]*?)```/g;
        let match;
        let stepIdx = 1;
        while ((match = codeBlockRegex.exec(reply.reply)) !== null) {
          const rawCode = match[1].trim();
          if (rawCode) {
            const lines = rawCode.split('\n').map(l => l.trim()).filter(l => l && !l.startsWith('#'));
            for (const line of lines) {
              stepsToPropose.push({
                title: `Jalankan: ${line.slice(0, 40)}${line.length > 40 ? '...' : ''}`,
                description: `Perintah diekstrak dari respon AI (Langkah ${stepIdx})`,
                command: line,
                risk: 'low'
              });
              stepIdx++;
            }
          }
        }
      }

      if (stepsToPropose.length > 0) {
        proposedSteps = stepsToPropose;
        acceptedSteps = stepsToPropose.map(() => true);
        runs = stepsToPropose.map(() => ({ status: 'pending', output: '' }));
      }
    } catch (err) {
      const errStr = errorText(err);
      if (errStr.includes('disabled') || errStr.includes('AiMode::Off') || errStr.includes('AI module is disabled')) {
        aiDisabledWarning = true;
        messages = [
          ...messages,
          {
            role: 'assistant',
            content: '⚠️ Modul AI saat ini berstatus nonaktif. Silakan pilih penyedia AI (OpenAI / Ollama / Custom API) di menu Pengaturan.'
          }
        ];
      } else {
        errorMsg = errStr;
        messages = [
          ...messages,
          {
            role: 'assistant',
            content: `Maaf, terjadi kendala: ${errStr}`
          }
        ];
      }
    } finally {
      isSending = false;
      scrollToBottom();
    }
  }

  function confirmClearChat() {
    messages = [
      {
        role: 'assistant',
        content: initialGreeting
      }
    ];
    proposedSteps = [];
    acceptedSteps = [];
    runs = [];
    showConfirmClear = false;
    aiDisabledWarning = false;
    showToast('Riwayat percakapan berhasil dibersihkan', 'success');
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      if (showConfirmClear) {
        showConfirmClear = false;
      } else {
        onClose();
      }
    } else if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault();
      void send();
    }
  }

  async function executeStepItem(step: AiPlanStep, i: number, effectiveExecMode: ExecMode) {
    if (step.action_type === 'caterm_action') {
      runs[i] = { status: 'running', output: '' };
      try {
        const action = (step.action_name || '').toLowerCase();
        const params = step.action_params || {};

        if (action === 'create_host' || action === 'save_host' || action === 'add_host') {
          const label = params.label || params.name || params.host || 'New Host';
          const hostname = params.hostname || params.host || params.ip || '127.0.0.1';
          const port = Number(params.port) || 22;
          const username = params.username || params.user || 'root';
          const authMethod = params.auth_method || params.authMethod || 'Password';
          const groupName = params.group || params.group_name || params.groupName || '';
          
          await saveHost({
            id: params.id,
            label,
            hostname,
            port,
            username,
            auth_method: authMethod === 'Key' ? { Key: { key_path: params.key_path || '' } } : { Password: {} },
            group_name: groupName || undefined,
            tags: params.tags || []
          });
          window.dispatchEvent(new CustomEvent('caterm:hosts-updated'));
          runs[i] = { status: 'ok', output: `Host "${label}" (${username}@${hostname}:${port}) berhasil disimpan!` };
          showToast(`Host "${label}" disimpan!`, 'success');
          return true;
        }

        if (action === 'delete_host' && params.id) {
          await deleteHost(params.id);
          window.dispatchEvent(new CustomEvent('caterm:hosts-updated'));
          runs[i] = { status: 'ok', output: `Host berhasil dihapus!` };
          showToast(`Host dihapus!`, 'success');
          return true;
        }

        if (action === 'create_snippet' || action === 'save_snippet' || action === 'add_snippet') {
          const snippetCmd = params.command || params.cmd || step.command || '';
          const snippetTitle = params.label || params.title || step.title || 'New Snippet';
          const snippetDesc = params.description || params.desc || step.description || '';
          const snippetTags = params.tags || [];
          await saveSnippet({
            id: params.id,
            label: snippetTitle,
            command: snippetCmd,
            description: snippetDesc,
            tags: Array.isArray(snippetTags) ? snippetTags : [String(snippetTags)]
          });
          window.dispatchEvent(new CustomEvent('caterm:snippets-updated'));
          runs[i] = { status: 'ok', output: `Snippet "${snippetTitle}" berhasil disimpan!` };
          showToast(`Snippet "${snippetTitle}" disimpan!`, 'success');
          return true;
        }

        if (action === 'delete_snippet' && params.id) {
          await deleteSnippet(params.id);
          window.dispatchEvent(new CustomEvent('caterm:snippets-updated'));
          runs[i] = { status: 'ok', output: `Snippet berhasil dihapus!` };
          showToast(`Snippet dihapus!`, 'success');
          return true;
        }

        if (action === 'navigate' && params.route) {
          goto(params.route);
          runs[i] = { status: 'ok', output: `Navigasi ke ${params.route}` };
          return true;
        }
      } catch (err) {
        runs[i] = { status: 'failed', output: errorText(err) };
        showToast(`Aksi gagal: ${errorText(err)}`, 'error');
        return false;
      }
    }

    if (effectiveExecMode === 'terminal') {
      runs[i] = { status: 'running', output: '' };
      injectIntoActiveSession(step.command);
      runs[i] = { status: 'ok', output: t('aiChat.sentToTerminal') };
      return true;
    }

    runs[i] = { status: 'running', output: '' };
    try {
      const result = await aiExecuteStep(targetHostId || LOCAL_HOST_ID, step);
      const output = [result.stdout, result.stderr].filter(Boolean).join('\n').trim();
      runs[i] = {
        status: result.success ? 'ok' : 'failed',
        output: output || t('aiChat.noOutput'),
        exitCode: result.exit_code
      };
      if (!result.success) {
        showToast(t('aiChat.stepFailed', { step: i + 1, code: result.exit_code ?? '?' }), 'error');
        return false;
      }
      return true;
    } catch (err) {
      runs[i] = {
        status: 'failed',
        output: errorText(err)
      };
      return false;
    }
  }

  async function runPlan() {
    if (isExecuting) return;
    isExecuting = true;
    try {
      for (let i = 0; i < proposedSteps.length; i++) {
        if (!acceptedSteps[i]) continue;
        const step = proposedSteps[i];
        const success = await executeStepItem(step, i, execMode);
        if (!success) break;
      }
    } finally {
      isExecuting = false;
    }
  }

  const acceptedCount = $derived(acceptedSteps.filter(Boolean).length);
</script>

<svelte:window onkeydown={handleKeydown} />

{#if isMinimized}
  <!-- Floating collapsed bubble pill in bottom right corner -->
  <div class="fixed bottom-5 right-5 z-50 animate-in fade-in duration-150">
    <button
      onclick={() => (isMinimized = false)}
      class="flex items-center gap-2 px-3.5 py-2 rounded-full bg-rose-600 hover:bg-rose-500 text-white shadow-xl shadow-rose-600/30 font-medium text-xs border border-rose-400/30 transition-transform hover:scale-105 cursor-pointer"
    >
      <span class="w-2 h-2 rounded-full bg-emerald-400 animate-pulse"></span>
      <span>Hana AI Aktif</span>
      <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 15l7-7 7 7" />
      </svg>
    </button>
  </div>
{:else}
  <!-- Floating Smart Card Overlay (Non-Destructive) in bottom right corner -->
  <aside
    class="fixed z-50 flex flex-col bg-neutral-900/95 backdrop-blur-xl border border-neutral-700/80 rounded-2xl shadow-2xl shadow-black/80 overflow-hidden transition-all duration-200 {isFullHeight
      ? 'top-[calc(3.5rem+env(safe-area-inset-top,0px))] bottom-3 right-3 w-[440px]'
      : 'bottom-5 right-5 w-[420px] h-[560px] max-h-[calc(100vh-4.5rem-env(safe-area-inset-top,0px))]'}"
    aria-label={t('aiChat.title')}
  >
    <!-- Header -->
    <header class="px-4 py-3 border-b border-neutral-800/80 bg-neutral-950/70 flex items-center justify-between select-none shrink-0">
      <div class="flex items-center gap-2.5 min-w-0">
        <div class="w-2.5 h-2.5 rounded-full bg-rose-500 shadow-sm shadow-rose-500/50 shrink-0"></div>
        <div class="min-w-0">
          <h2 class="text-xs font-bold text-white tracking-wide truncate">Hana AI (DevOps Copilot)</h2>
        </div>
      </div>

      <div class="flex items-center gap-1 shrink-0">
        <!-- Clear Chat button -->
        <button
          type="button"
          onclick={() => (showConfirmClear = true)}
          class="p-1 text-neutral-400 hover:text-amber-400 rounded-lg hover:bg-neutral-800 transition-colors"
          title="Bersihkan riwayat percakapan"
          aria-label="Bersihkan Chat"
        >
          <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
          </svg>
        </button>

        <!-- Minimize button -->
        <button
          type="button"
          onclick={() => (isMinimized = true)}
          class="p-1 text-neutral-400 hover:text-white rounded-lg hover:bg-neutral-800 transition-colors"
          title="Kecilkan ke pojok"
          aria-label="Kecilkan"
        >
          <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M19 9l-7 7-7-7" />
          </svg>
        </button>

        <!-- Expand / Restore button -->
        <button
          type="button"
          onclick={() => (isFullHeight = !isFullHeight)}
          class="p-1 text-neutral-400 hover:text-white rounded-lg hover:bg-neutral-800 transition-colors"
          title={isFullHeight ? "Mode Kartu Mengambang" : "Mode Layar Penuh"}
          aria-label="Ubah Ukuran"
        >
          <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            {#if isFullHeight}
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5l-5-5m5 5v-4m0 4h-4" />
            {:else}
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 3H5a2 2 0 00-2 2v3m18 0V5a2 2 0 00-2-2h-3m0 18h3a2 2 0 002-2v-3M3 16v3a2 2 0 002 2h3" />
            {/if}
          </svg>
        </button>

        <!-- Close button -->
        <button
          type="button"
          onclick={onClose}
          class="p-1 text-neutral-400 hover:text-rose-400 rounded-lg hover:bg-neutral-800 transition-colors"
          title="Tutup Hana AI"
          aria-label="Tutup"
        >
          <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>
    </header>

    <!-- Clear Chat Confirmation Overlay -->
    {#if showConfirmClear}
      <div class="p-4 bg-amber-500/10 border-b border-amber-500/20 text-xs animate-in fade-in duration-150">
        <p class="font-semibold text-amber-300 mb-1">Bersihkan riwayat percakapan?</p>
        <p class="text-neutral-400 mb-3 text-[11px]">Tindakan ini akan mengosongkan seluruh log pesan terminal AI.</p>
        <div class="flex justify-end gap-2">
          <button
            type="button"
            onclick={() => (showConfirmClear = false)}
            class="px-2.5 py-1 rounded-lg bg-neutral-800 hover:bg-neutral-700 text-neutral-300 text-xs transition-colors"
          >
            Batal
          </button>
          <button
            type="button"
            onclick={confirmClearChat}
            class="px-2.5 py-1 rounded-lg bg-amber-600 hover:bg-amber-500 text-white text-xs font-semibold transition-colors"
          >
            Ya, Bersihkan
          </button>
        </div>
      </div>
    {/if}

    <!-- Terminal Controls: Target Host & Execution Mode -->
    <div class="px-3.5 py-2.5 border-b border-neutral-800/80 bg-neutral-950/40 flex items-center gap-2.5 shrink-0 text-xs">
      <div class="flex-1 flex items-center gap-1.5 min-w-0">
        <span class="text-[10px] font-semibold text-neutral-400 shrink-0">Host:</span>
        <select
          bind:value={targetHostId}
          class="flex-1 min-w-0 bg-neutral-900 border border-neutral-700/70 rounded-lg px-2 py-1 text-[11px] text-white focus:outline-none focus:border-rose-500 cursor-pointer truncate"
        >
          <option value={LOCAL_HOST_ID}>🖥️ Local Terminal (localhost)</option>
          {#each hosts as host (host.id)}
            <option value={host.id}>🌐 {host.label} ({host.username}@{host.hostname})</option>
          {/each}
        </select>
      </div>

      <div class="flex items-center gap-1.5 shrink-0">
        <span class="text-[10px] font-semibold text-neutral-400">Via:</span>
        <div class="inline-flex rounded-lg bg-neutral-900 p-0.5 border border-neutral-800">
          <button
            type="button"
            onclick={() => (execMode = 'ssh')}
            class="px-2 py-0.5 rounded text-[10px] font-medium transition-colors {execMode === 'ssh'
              ? 'bg-rose-600 text-white shadow-sm'
              : 'text-neutral-400 hover:text-white'}"
          >
            SSH
          </button>
          <button
            type="button"
            onclick={() => (execMode = 'terminal')}
            class="px-2 py-0.5 rounded text-[10px] font-medium transition-colors {execMode === 'terminal'
              ? 'bg-rose-600 text-white shadow-sm'
              : 'text-neutral-400 hover:text-white'}"
          >
            Terminal
          </button>
        </div>
      </div>
    </div>

    <!-- Messages Container -->
    <div bind:this={scroller} class="flex-1 overflow-y-auto p-4 space-y-3 text-xs leading-relaxed">
      {#each messages as msg}
        <div class="flex flex-col {msg.role === 'user' ? 'items-end' : 'items-start'}">
          <div
            class="max-w-[90%] rounded-2xl px-3.5 py-2.5 {msg.role === 'user'
              ? 'bg-rose-600 text-white rounded-br-none shadow-md shadow-rose-600/20'
              : 'bg-neutral-800/90 text-neutral-200 rounded-bl-none border border-neutral-700/60'}"
          >
            {cleanMessageContent(msg.content)}
          </div>
        </div>
      {/each}

      <!-- Step Approval Section -->
      {#if proposedSteps.length > 0}
        <div class="p-3 bg-neutral-950/80 border border-neutral-700/80 rounded-xl space-y-2 mt-2">
          <div class="flex items-center justify-between text-[11px] font-semibold text-neutral-300">
            <span>Rencana Eksekusi ({proposedSteps.length} langkah)</span>
            <button
              onclick={runPlan}
              disabled={isExecuting || acceptedCount === 0}
              class="px-2.5 py-1 rounded-lg bg-rose-600 hover:bg-rose-500 disabled:opacity-50 text-white text-xs font-bold transition-all shadow-sm"
            >
              {isExecuting ? 'Mengeksekusi...' : `Eksekusi (${acceptedCount})`}
            </button>
          </div>

          <div class="space-y-1.5">
            {#each proposedSteps as step, i}
              <div class="p-2 rounded-lg bg-neutral-900 border border-neutral-800 text-[11px] space-y-1">
                <div class="flex items-start gap-2">
                  <input
                    type="checkbox"
                    bind:checked={acceptedSteps[i]}
                    class="mt-0.5 rounded border-neutral-700 text-rose-600 focus:ring-0"
                  />
                  <div class="flex-1 min-w-0">
                    <div class="font-medium text-neutral-200">{step.title}</div>
                    {#if step.command}
                      <code class="block mt-1 p-1 bg-black/60 rounded text-[10px] text-rose-300 font-mono break-all">
                        {step.command}
                      </code>
                    {/if}
                    {#if runs[i]?.output}
                      <div class="mt-1 text-[10px] {runs[i].status === 'ok' ? 'text-emerald-400' : 'text-rose-400'} font-mono bg-neutral-950 p-1 rounded">
                        {runs[i].output}
                      </div>
                    {/if}
                  </div>
                </div>
              </div>
            {/each}
          </div>
        </div>
      {/if}

      <!-- AI Disabled Warning -->
      {#if aiDisabledWarning}
        <div class="p-3.5 rounded-xl bg-rose-500/10 border border-rose-500/30 flex flex-col gap-2">
          <div class="flex items-center gap-2 text-rose-400 font-semibold text-xs">
            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
            <span>Konfigurasi AI Provider</span>
          </div>
          <p class="text-[11px] text-neutral-400">Pilih mode "BYO" dan masukkan API Key / URL endpoint AI Anda.</p>
          <button
            type="button"
            onclick={() => {
              onClose();
              goto('/settings?tab=ai');
            }}
            class="w-full py-2 rounded-lg bg-rose-600 hover:bg-rose-500 text-white font-semibold text-xs shadow-md shadow-rose-600/20 transition-all text-center"
          >
            Buka Pengaturan AI
          </button>
        </div>
      {/if}

      {#if isSending}
        <div class="flex items-center gap-2 text-neutral-400 text-xs py-2">
          <svg class="w-3.5 h-3.5 animate-spin text-rose-400" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          <span>Menghubungi Hana AI...</span>
        </div>
      {/if}
    </div>

    <!-- Input Composer -->
    <div class="p-3 border-t border-neutral-800/80 bg-neutral-950/80 shrink-0">
      <form
        onsubmit={(e) => {
          e.preventDefault();
          void send();
        }}
        class="flex gap-2"
      >
        <input
          type="text"
          bind:value={draft}
          placeholder="Tanyakan perintah Linux, troubleshoot, atau script..."
          class="flex-1 px-3.5 py-2 bg-neutral-900 border border-neutral-700/70 rounded-xl text-xs text-white placeholder-neutral-500 focus:outline-none focus:border-rose-500 transition-colors"
        />
        <button
          type="submit"
          disabled={isSending || !draft.trim()}
          class="px-3.5 py-2 bg-rose-600 hover:bg-rose-500 disabled:opacity-40 text-white text-xs font-semibold rounded-xl transition-all shadow-md shadow-rose-600/20 shrink-0"
        >
          Kirim
        </button>
      </form>
    </div>
  </aside>
{/if}
