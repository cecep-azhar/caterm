<script lang="ts">
  import Button from '$lib/components/ui/Button.svelte';
  import Badge from '$lib/components/ui/Badge.svelte';
  import Icon from '$lib/components/ui/Icon.svelte';

  export interface ChatMessage {
    id: string;
    role: 'user' | 'assistant' | 'system';
    content: string;
    timestamp: string;
    codeSnippet?: string;
    suggestedActions?: string[];
  }

  interface Props {
    title?: string;
    copilotName?: string;
    class?: string;
  }

  let {
    title = 'AI Terminal Co-pilot',
    copilotName = 'Hermes DevOps v3',
    class: customClass = '',
  }: Props = $props();

  let messages = $state<ChatMessage[]>([
    {
      id: '1',
      role: 'assistant',
      content: 'Hello! I am your CADS DevOps assistant. I can help analyze failed systemd units, explain Docker log traces, or construct optimized bash pipelines.',
      timestamp: '10:42 AM',
      suggestedActions: [
        'Check systemd error logs',
        'Find large files over 500MB',
        'Inspect listening TCP ports',
      ],
    },
    {
      id: '2',
      role: 'user',
      content: 'How do I check which process is holding port 8080 and restart nginx safely?',
      timestamp: '10:43 AM',
    },
    {
      id: '3',
      role: 'assistant',
      content: 'You can check port 8080 listeners using `lsof` or `ss`, then run a zero-downtime reload of nginx.',
      timestamp: '10:43 AM',
      codeSnippet: `# Find process bound to port 8080
sudo ss -tulpn | grep :8080

# Or using lsof
sudo lsof -i :8080

# Verify nginx syntax and test reload
sudo nginx -t && sudo systemctl reload nginx`,
      suggestedActions: ['Execute command in terminal', 'Explain ss flags'],
    },
  ]);

  let inputPrompt = $state('');
  let isGenerating = $state(false);

  function handleSend() {
    if (!inputPrompt.trim() || isGenerating) return;

    const userText = inputPrompt;
    messages = [
      ...messages,
      {
        id: Date.now().toString(),
        role: 'user',
        content: userText,
        timestamp: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
      },
    ];
    inputPrompt = '';
    isGenerating = true;

    // Simulate AI response
    setTimeout(() => {
      messages = [
        ...messages,
        {
          id: (Date.now() + 1).toString(),
          role: 'assistant',
          content: `I've analyzed your prompt regarding "${userText}". Here is the recommended CADS execution step:`,
          timestamp: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
          codeSnippet: `# Generated shell diagnostic\nsudo journalctl -u nginx.service -n 50 --no-pager`,
          suggestedActions: ['Copy output', 'Run in current SSH pane'],
        },
      ];
      isGenerating = false;
    }, 900);
  }

  function handleActionClick(actionText: string) {
    inputPrompt = actionText;
    handleSend();
  }
</script>

<div class="flex flex-col h-[560px] w-full rounded-2xl bg-[#0E0E12] border border-[#272732] text-[#EDEDED] font-sans shadow-xl overflow-hidden {customClass}">
  <!-- Co-pilot Header -->
  <div class="px-4 py-3 bg-[#121217] border-b border-[#272732] flex items-center justify-between">
    <div class="flex items-center gap-2.5">
      <div class="w-7 h-7 rounded-lg bg-[var(--ca-brand)]/20 border border-[var(--ca-brand)]/40 flex items-center justify-center text-[var(--ca-brand)] shadow-xs">
        <Icon name="sparkles" size={14} />
      </div>
      <div>
        <div class="flex items-center gap-2">
          <h3 class="text-xs font-bold text-white">{title}</h3>
          <Badge variant="brand" size="sm">{copilotName}</Badge>
        </div>
        <span class="text-[10px] text-neutral-400">Context aware · Remote host: prod-api-cluster-01</span>
      </div>
    </div>

    <div class="flex items-center gap-1">
      <button
        type="button"
        onclick={() => (messages = [messages[0]])}
        class="p-1.5 text-neutral-400 hover:text-white rounded-lg hover:bg-neutral-800 transition cursor-pointer"
        title="Clear conversation"
      >
        <Icon name="refresh" size={13} />
      </button>
    </div>
  </div>

  <!-- Message Stream -->
  <div class="flex-1 overflow-y-auto p-4 space-y-4">
    {#each messages as msg (msg.id)}
      <div class="flex flex-col {msg.role === 'user' ? 'items-end' : 'items-start'} space-y-1.5">
        <div class="flex items-center gap-2 px-1 text-[10px] text-neutral-400">
          <span class="font-semibold">{msg.role === 'user' ? 'You (DevOps)' : copilotName}</span>
          <span>{msg.timestamp}</span>
        </div>

        <div
          class="max-w-[85%] rounded-xl p-3 text-xs leading-relaxed {msg.role === 'user'
            ? 'bg-[var(--ca-brand)]/20 border border-[var(--ca-brand)]/40 text-white rounded-tr-none'
            : 'bg-[#18181F] border border-[#272732] text-neutral-200 rounded-tl-none'}"
        >
          <p>{msg.content}</p>

          <!-- Code Snippet Block if present -->
          {#if msg.codeSnippet}
            <div class="mt-2.5 rounded-lg bg-[#0A0A0C] border border-[#272732] overflow-hidden font-mono text-[11px]">
              <div class="px-3 py-1 bg-[#121217] border-b border-[#272732] flex items-center justify-between text-neutral-400 text-[10px]">
                <span>bash</span>
                <button
                  type="button"
                  class="hover:text-white flex items-center gap-1 cursor-pointer"
                  onclick={() => navigator?.clipboard?.writeText(msg.codeSnippet || '')}
                >
                  <Icon name="copy" size={10} />
                  Copy
                </button>
              </div>
              <pre class="p-3 text-cyan-300 overflow-x-auto whitespace-pre-wrap">{msg.codeSnippet}</pre>
            </div>
          {/if}

          <!-- Action suggestions chips -->
          {#if msg.suggestedActions && msg.suggestedActions.length > 0}
            <div class="mt-3 pt-2 border-t border-[#272732]/60 flex flex-wrap gap-1.5">
              {#each msg.suggestedActions as act}
                <button
                  type="button"
                  onclick={() => handleActionClick(act)}
                  class="px-2 py-1 rounded-md bg-[#121217] hover:bg-[#20202A] border border-[#272732] text-[10px] text-[var(--ca-brand)] hover:text-white transition cursor-pointer"
                >
                  ⚡ {act}
                </button>
              {/each}
            </div>
          {/if}
        </div>
      </div>
    {/each}

    {#if isGenerating}
      <div class="flex items-center gap-2 text-xs text-neutral-400 italic pl-1">
        <span class="w-2 h-2 rounded-full bg-[var(--ca-brand)] animate-ping"></span>
        Hermes Co-pilot is generating response...
      </div>
    {/if}
  </div>

  <!-- Input Prompt Toolbar -->
  <div class="p-3 bg-[#121217] border-t border-[#272732]">
    <div class="flex items-end gap-2 bg-[#0A0A0C] border border-[#272732] rounded-xl p-2 focus-within:border-[var(--ca-brand)]/60 transition-colors">
      <textarea
        bind:value={inputPrompt}
        rows="2"
        placeholder="Ask co-pilot to generate command, diagnose incident, or write script..."
        onkeydown={(e) => {
          if (e.key === 'Enter' && !e.shiftKey) {
            e.preventDefault();
            handleSend();
          }
        }}
        class="flex-1 bg-transparent border-0 outline-none text-xs text-white placeholder-neutral-500 resize-none font-sans"
      ></textarea>

      <Button
        variant="brand"
        size="sm"
        disabled={!inputPrompt.trim() || isGenerating}
        onclick={handleSend}
        class="shrink-0 cursor-pointer"
      >
        <Icon name="sparkles" size={12} />
        Send
      </Button>
    </div>
  </div>
</div>
