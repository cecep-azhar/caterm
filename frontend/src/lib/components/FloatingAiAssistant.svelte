<script lang="ts">
  import { onMount } from 'svelte';
  import {
    aiGetSkills,
    aiGetRoutingMatrix,
    aiDispatchTaskWithSkill,
    type CustomSkill,
    type TaskRouteRule,
    type DispatchResult
  } from '$lib/api/aiRouting';
  import { showToast } from '$lib/stores/uiNotifications.svelte';
  import { errorText } from '$lib/errors';

  let isOpen = $state(false);
  let promptInput = $state('');
  let isSending = $state(false);
  let skills = $state<CustomSkill[]>([]);
  let activeRule = $state<TaskRouteRule | null>(null);
  let messages = $state<Array<{ role: 'user' | 'assistant'; text: string; skill?: string; model?: string }>>([]);

  // Autocomplete state for `@`
  let showSkillSuggestions = $state(false);
  let skillFilter = $state('');
  let selectedSkillIndex = $state(0);

  const filteredSkills = $derived(
    skills.filter((s) => s.is_enabled && (s.name.toLowerCase().includes(skillFilter.toLowerCase()) || s.title.toLowerCase().includes(skillFilter.toLowerCase())))
  );

  onMount(async () => {
    try {
      const [allSkills, matrix] = await Promise.all([
        aiGetSkills().catch(() => []),
        aiGetRoutingMatrix().catch(() => [])
      ]);
      skills = allSkills;
      activeRule = matrix.find((r) => r.task_type === 'chat') || null;
    } catch {
      // Ignore background fetch failure
    }
  });

  function handleInput(e: Event) {
    const target = e.target as HTMLTextAreaElement | HTMLInputElement;
    const val = target.value;
    promptInput = val;

    const lastAtPos = val.lastIndexOf('@');
    if (lastAtPos !== -1 && lastAtPos === val.length - 1) {
      showSkillSuggestions = true;
      skillFilter = '';
      selectedSkillIndex = 0;
    } else if (lastAtPos !== -1 && lastAtPos < val.length) {
      const query = val.slice(lastAtPos + 1);
      if (!query.includes(' ')) {
        showSkillSuggestions = true;
        skillFilter = query;
      } else {
        showSkillSuggestions = false;
      }
    } else {
      showSkillSuggestions = false;
    }
  }

  function pickSkill(skill: CustomSkill) {
    const lastAtPos = promptInput.lastIndexOf('@');
    if (lastAtPos !== -1) {
      promptInput = promptInput.slice(0, lastAtPos) + `@${skill.name} `;
    } else {
      promptInput = `@${skill.name} ` + promptInput;
    }
    showSkillSuggestions = false;
  }

  async function handleSend() {
    const text = promptInput.trim();
    if (!text || isSending) return;

    // Detect if prompt starts with @skill-name
    let detectedSkill: string | null = null;
    const match = text.match(/^@([a-zA-Z0-9_-]+)/);
    if (match) {
      detectedSkill = match[1];
    }

    messages = [...messages, { role: 'user', text, skill: detectedSkill ?? undefined }];
    promptInput = '';
    showSkillSuggestions = false;
    isSending = true;

    try {
      const res: DispatchResult = await aiDispatchTaskWithSkill('chat', text, detectedSkill);
      messages = [
        ...messages,
        {
          role: 'assistant',
          text: res.content,
          model: res.model,
          skill: detectedSkill ?? undefined
        }
      ];
    } catch (err) {
      messages = [
        ...messages,
        {
          role: 'assistant',
          text: `⚠️ Error executing prompt: ${errorText(err)}`
        }
      ];
      showToast(errorText(err), 'error');
    } finally {
      isSending = false;
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (showSkillSuggestions && filteredSkills.length > 0) {
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        selectedSkillIndex = (selectedSkillIndex + 1) % filteredSkills.length;
        return;
      }
      if (e.key === 'ArrowUp') {
        e.preventDefault();
        selectedSkillIndex = (selectedSkillIndex - 1 + filteredSkills.length) % filteredSkills.length;
        return;
      }
      if (e.key === 'Enter' || e.key === 'Tab') {
        e.preventDefault();
        pickSkill(filteredSkills[selectedSkillIndex]);
        return;
      }
      if (e.key === 'Escape') {
        showSkillSuggestions = false;
        return;
      }
    }

    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      void handleSend();
    }
  }
</script>

<!-- FLOATING CONTAINER -->
<div class="fixed bottom-4 right-4 z-50 select-none">
  {#if !isOpen}
    <!-- Collapsed Trigger Pill -->
    <button
      type="button"
      onclick={() => (isOpen = true)}
      class="flex items-center gap-2.5 px-4 py-2.5 rounded-full bg-neutral-900/95 hover:bg-neutral-800 text-neutral-100 border border-neutral-700/80 shadow-2xl shadow-black/60 hover:scale-105 active:scale-95 transition-all text-xs font-semibold cursor-pointer group"
      title="Hana AI Master Router & Skills Assistant"
    >
      <span class="w-2 h-2 rounded-full bg-emerald-500 animate-pulse"></span>
      <svg class="w-4 h-4 text-rose-500 group-hover:rotate-12 transition-transform shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.813 15.904L9 18.75l-.813-2.846a4.5 4.5 0 00-3.09-3.09L2.25 12l2.846-.813a4.5 4.5 0 003.09-3.09L9 5.25l.813 2.846a4.5 4.5 0 003.09 3.09L15.75 12l-2.846.813a4.5 4.5 0 00-3.09 3.09zM18.259 8.715L18 9.75l-.259-1.035a3.375 3.375 0 00-2.455-2.456L14.25 6l1.036-.259a3.375 3.375 0 002.455-2.456L18 2.25l.259 1.035a3.375 3.375 0 002.456 2.456L21.75 6l-1.035.259a3.375 3.375 0 00-2.456 2.456z" />
      </svg>
      <span>Hana AI</span>
      {#if activeRule}
        <span class="text-[10px] px-1.5 py-0.5 rounded bg-sky-500/20 text-sky-400 font-mono">
          {activeRule.primary_model}
        </span>
      {/if}
    </button>
  {:else}
    <!-- Expanded Floating Smart Card -->
    <div class="w-[380px] sm:w-[440px] h-[520px] flex flex-col rounded-2xl bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 shadow-2xl shadow-black/40 overflow-hidden animate-in fade-in zoom-in-95 duration-150">
      <!-- Card Header -->
      <div class="px-4 py-3 bg-neutral-50 dark:bg-neutral-950/80 border-b border-neutral-200 dark:border-neutral-800 flex items-center justify-between">
        <div class="flex items-center gap-2">
          <div class="w-2 h-2 rounded-full bg-emerald-500"></div>
          <span class="text-xs font-bold text-neutral-900 dark:text-white">Hana AI Assistant</span>
          {#if activeRule}
            <span class="text-[10px] px-1.5 py-0.5 rounded bg-sky-500/20 text-sky-600 dark:text-sky-400 font-mono">
              {activeRule.primary_model}
            </span>
          {/if}
        </div>
        <div class="flex items-center gap-1">
          <button
            type="button"
            onclick={() => (messages = [])}
            class="text-[11px] px-2 py-1 text-neutral-400 hover:text-neutral-700 dark:hover:text-neutral-200 transition-colors"
            title="Clear Chat"
          >
            Clear
          </button>
          <button
            type="button"
            onclick={() => (isOpen = false)}
            class="w-6 h-6 flex items-center justify-center rounded-lg text-neutral-400 hover:text-neutral-700 dark:hover:text-white hover:bg-neutral-200 dark:hover:bg-neutral-800 transition-colors"
            title="Minimize"
          >
            ✕
          </button>
        </div>
      </div>

      <!-- Quick Skills Pills -->
      <div class="px-3 py-1.5 bg-neutral-100/50 dark:bg-neutral-950/40 border-b border-neutral-200/50 dark:border-neutral-800/50 flex items-center gap-1.5 overflow-x-auto text-[11px]">
        <span class="text-neutral-400 shrink-0 text-[10px] uppercase font-bold">Skills:</span>
        {#each skills.slice(0, 4) as s}
          <button
            type="button"
            onclick={() => pickSkill(s)}
            class="px-2 py-0.5 rounded-full bg-neutral-200/60 dark:bg-neutral-800 text-neutral-700 dark:text-neutral-300 hover:bg-sky-500/20 hover:text-sky-400 transition-colors whitespace-nowrap font-mono text-[10px]"
          >
            @{s.name}
          </button>
        {/each}
      </div>

      <!-- Messages Stream -->
      <div class="flex-1 p-4 overflow-y-auto space-y-3 text-xs">
        {#if messages.length === 0}
          <div class="h-full flex flex-col items-center justify-center text-center text-neutral-400 space-y-2">
            <svg class="w-8 h-8 text-neutral-500 opacity-60" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
            </svg>
            <p class="font-medium text-neutral-600 dark:text-neutral-300">Tanya Hana AI atau ketik <code class="text-sky-500 font-mono">@</code> untuk memanggil skill.</p>
            <p class="text-[11px] text-neutral-500 max-w-xs">Contoh: <code>@k8s-triage analisa pod crash di production</code></p>
          </div>
        {/if}

        {#each messages as m}
          <div class="flex flex-col {m.role === 'user' ? 'items-end' : 'items-start'}">
            {#if m.skill}
              <span class="text-[9px] font-mono text-sky-500 mb-0.5 px-1 rounded bg-sky-500/10">@{m.skill}</span>
            {/if}
            <div class="max-w-[88%] p-3 rounded-2xl {m.role === 'user' ? 'bg-sky-600 text-white rounded-br-xs' : 'bg-neutral-100 dark:bg-neutral-800 text-neutral-900 dark:text-neutral-100 rounded-bl-xs border border-neutral-200/60 dark:border-neutral-700/60'} whitespace-pre-wrap font-sans text-xs">
              {m.text}
            </div>
            {#if m.model}
              <span class="text-[9px] font-mono text-neutral-400 mt-0.5">{m.model}</span>
            {/if}
          </div>
        {/each}

        {#if isSending}
          <div class="flex items-center gap-2 text-neutral-400 text-xs py-2">
            <span class="w-2 h-2 rounded-full bg-sky-500 animate-ping"></span>
            <span>Hana AI sedang berpikir...</span>
          </div>
        {/if}
      </div>

      <!-- Autocomplete Dropdown Popup -->
      {#if showSkillSuggestions && filteredSkills.length > 0}
        <div class="mx-3 mb-1 p-1 bg-white dark:bg-neutral-950 border border-neutral-200 dark:border-neutral-800 rounded-xl shadow-xl max-h-40 overflow-y-auto text-xs z-30">
          <div class="px-2 py-1 text-[10px] uppercase font-bold text-neutral-400">Pilih Custom Skill:</div>
          {#each filteredSkills as s, idx}
            <button
              type="button"
              onclick={() => pickSkill(s)}
              class="w-full text-left px-2.5 py-1.5 rounded-lg flex items-center justify-between gap-2 {idx === selectedSkillIndex ? 'bg-sky-500/20 text-sky-400' : 'hover:bg-neutral-100 dark:hover:bg-neutral-800 text-neutral-700 dark:text-neutral-200'}"
            >
              <span class="font-mono font-bold text-xs">@{s.name}</span>
              <span class="text-[11px] text-neutral-400 truncate">{s.title}</span>
            </button>
          {/each}
        </div>
      {/if}

      <!-- Input Bar -->
      <div class="p-3 bg-neutral-50 dark:bg-neutral-950/80 border-t border-neutral-200 dark:border-neutral-800">
        <div class="flex items-end gap-2 bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-xl px-3 py-2 focus-within:border-sky-500 transition-colors shadow-inner">
          <textarea
            rows="2"
            value={promptInput}
            oninput={handleInput}
            onkeydown={handleKeyDown}
            placeholder="Tanya Hana AI (ketik @ untuk skill)..."
            class="flex-1 bg-transparent text-xs text-neutral-900 dark:text-white placeholder-neutral-400 resize-none focus:outline-none"
          ></textarea>
          <button
            type="button"
            onclick={handleSend}
            disabled={isSending || !promptInput.trim()}
            class="p-1.5 bg-sky-600 hover:bg-sky-500 disabled:opacity-40 text-white rounded-lg transition-colors shrink-0"
            title="Kirim (Enter)"
          >
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14 5l7 7m0 0l-7 7m7-7H3"/></svg>
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>
