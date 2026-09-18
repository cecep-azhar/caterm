<script lang="ts">
  import { injectIntoActiveSession, hasActiveSession } from '$lib/stores/activeSession.svelte';

  let isAddModalOpen = $state(false);
  let creationStep = $state(1); // 1: command, 2: details
  let injectNotice = $state('');
  
  let snippets = $state([
    { id: 1, label: "Docker Clean All", desc: "Stops and removes all containers, networks, and untagged images", cmd: "docker system prune -a --volumes -f", tags: ["docker", "cleanup"] },
    { id: 2, label: "Check System Logs", desc: "View the end of journalctl system log", cmd: "journalctl -xe -n 100", tags: ["linux", "debug"] }
  ]);

  let newSnippet = $state({
    cmd: "",
    label: "",
    desc: "",
    tags: ""
  });

  function resetModal() {
    newSnippet = { cmd: "", label: "", desc: "", tags: "" };
    creationStep = 1;
    isAddModalOpen = false;
  }

  function nextStep() {
    if (newSnippet.cmd.trim()) creationStep = 2;
  }

  function saveSnippet(e: Event) {
    e.preventDefault();
    if (!newSnippet.label) return;
    
    snippets.push({
      id: Date.now(),
      label: newSnippet.label,
      desc: newSnippet.desc,
      cmd: newSnippet.cmd,
      tags: newSnippet.tags.split(',').map(t => t.trim()).filter(Boolean)
    });

    resetModal();
  }

  function injectSnippet(cmd: string) {
    const injected = injectIntoActiveSession(cmd);
    injectNotice = injected
      ? 'Snippet dikirim ke sesi terminal aktif.'
      : 'Tidak ada sesi terminal aktif — buka sebuah sesi di tab Session dulu.';
    setTimeout(() => (injectNotice = ''), 3000);
  }
</script>

<div class="max-w-5xl mx-auto space-y-6">
  <div class="flex justify-between items-center">
    <div>
      <h1 class="text-2xl font-bold text-white tracking-tight">Command Snippets</h1>
      <p class="text-neutral-400 text-sm mt-1">Save reusable commands to inject instantly into active terminal sessions.</p>
    </div>
    <button 
      onclick={() => { resetModal(); isAddModalOpen = true; }}
      class="px-4 py-2 bg-sky-600 hover:bg-sky-500 text-white font-medium text-sm rounded-md transition-colors flex items-center gap-2">
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"></path></svg>
      Add Snippet
    </button>
  </div>

  {#if injectNotice}
    <div class="px-4 py-2 bg-neutral-900 border border-neutral-800 rounded text-xs text-sky-400">{injectNotice}</div>
  {/if}

  <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
    {#each snippets as snip}
      <div class="bg-neutral-900 border border-neutral-800 rounded-lg overflow-hidden group">
        <div class="p-4 border-b border-neutral-800">
          <div class="flex justify-between items-start">
            <h3 class="font-semibold text-white">{snip.label}</h3>
            <button
              onclick={() => injectSnippet(snip.cmd)}
              disabled={!hasActiveSession()}
              class="text-sky-400 hover:text-sky-300 px-2 py-1 bg-sky-500/10 hover:bg-sky-500/20 disabled:opacity-40 disabled:cursor-not-allowed rounded text-xs transition-colors">Inject</button>
          </div>
          <p class="text-neutral-400 text-xs mt-1">{snip.desc}</p>
        </div>
        <div class="bg-neutral-950 p-3 font-mono text-sm text-neutral-300 relative">
          <span class="opacity-50 select-none mr-2">$</span>{snip.cmd}
        </div>
        <div class="px-4 py-2 bg-neutral-900 flex gap-1 border-t border-neutral-800">
          {#each snip.tags as tag}
            <span class="px-2 py-0.5 bg-neutral-800 border border-neutral-700 text-neutral-400 rounded text-xs">{tag}</span>
          {/each}
        </div>
      </div>
    {/each}
  </div>
</div>

<!-- Create Snippet Modal (2 Steps) -->
{#if isAddModalOpen}
  <div class="fixed inset-0 bg-black/70 flex items-center justify-center p-4 z-50">
    <div class="bg-neutral-900 border border-neutral-800 rounded-lg p-6 max-w-lg w-full space-y-4">
      <div class="flex justify-between items-center border-b border-neutral-800 pb-3">
        <h3 class="text-xl font-bold text-white">Add Snippet</h3>
        <span class="text-xs text-neutral-500 font-medium bg-neutral-800 px-2 py-1 rounded">Step {creationStep} of 2</span>
      </div>
      
      {#if creationStep === 1}
        <div class="space-y-4">
          <div>
            <label for="snip-cmd" class="block text-xs font-medium text-neutral-400 uppercase mb-2">Bash Command</label>
            <textarea 
              id="snip-cmd" 
              bind:value={newSnippet.cmd} 
              rows="4" 
              placeholder="docker-compose up -d --build" 
              class="w-full px-3 py-2 bg-neutral-950 font-mono border border-neutral-800 rounded text-sm text-white focus:outline-none focus:border-sky-500"></textarea>
          </div>
          <div class="flex justify-end gap-2 pt-2 border-t border-neutral-800">
            <button 
              onclick={resetModal} 
              class="px-4 py-2 bg-neutral-800 hover:bg-neutral-700 text-neutral-300 rounded text-sm font-medium">Cancel</button>
            <button 
              onclick={nextStep}
              disabled={!newSnippet.cmd.trim()}
              class="px-4 py-2 bg-sky-600 hover:bg-sky-500 disabled:opacity-50 text-white rounded text-sm font-medium">Next</button>
          </div>
        </div>
      {:else}
        <form onsubmit={saveSnippet} class="space-y-4">
          <div>
            <label for="snip-label" class="block text-xs font-medium text-neutral-400 uppercase mb-1">Label</label>
            <input id="snip-label" bind:value={newSnippet.label} required placeholder="Restart Docker Stack" class="w-full px-3 py-2 bg-neutral-950 border border-neutral-800 rounded text-sm text-white focus:outline-none focus:border-sky-500" />
          </div>
          
          <div>
            <label for="snip-desc" class="block text-xs font-medium text-neutral-400 uppercase mb-1">Description (Optional)</label>
            <input id="snip-desc" bind:value={newSnippet.desc} placeholder="Rebuilds and restarts the web tier" class="w-full px-3 py-2 bg-neutral-950 border border-neutral-800 rounded text-sm text-white focus:outline-none focus:border-sky-500" />
          </div>
          
          <div>
            <label for="snip-tags" class="block text-xs font-medium text-neutral-400 uppercase mb-1">Tags (comma separated)</label>
            <input id="snip-tags" bind:value={newSnippet.tags} placeholder="docker, devops, deploy" class="w-full px-3 py-2 bg-neutral-950 border border-neutral-800 rounded text-sm text-white focus:outline-none focus:border-sky-500" />
          </div>

          <div class="flex justify-between items-center pt-2 border-t border-neutral-800">
            <button 
              type="button"
              onclick={() => creationStep = 1} 
              class="text-neutral-400 hover:text-white text-sm font-medium transition-colors">← Back</button>
            <div class="flex gap-2">
              <button 
                type="button"
                onclick={resetModal} 
                class="px-4 py-2 bg-neutral-800 hover:bg-neutral-700 text-neutral-300 rounded text-sm font-medium">Cancel</button>
              <button 
                type="submit"
                class="px-4 py-2 bg-emerald-600 hover:bg-emerald-500 text-white rounded text-sm font-medium">Save Snippet</button>
            </div>
          </div>
        </form>
      {/if}
    </div>
  </div>
{/if}