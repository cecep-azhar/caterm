<script lang="ts">
  import "../app.css";
  import LockScreen from '$lib/components/LockScreen.svelte';
  import NotificationCenter from '$lib/components/NotificationCenter.svelte';
  import Logo from '$lib/components/Logo.svelte';

  let { children } = $props();

  let isUnlocked = $state(false);
</script>

<NotificationCenter />

{#if !isUnlocked}
  <LockScreen onUnlocked={() => isUnlocked = true} />
{:else}
<div class="flex h-screen bg-[#0e0e0e] text-neutral-300 font-sans">
  <!-- Sidebar -->
  <aside class="w-16 md:w-64 border-r border-neutral-800 flex flex-col justify-between shrink-0">
    <div>
      <div class="h-12 border-b border-neutral-800 flex items-center justify-between px-4">
        <div class="flex items-center gap-2">
          <Logo size={24} mode="dark" />
          <span class="font-bold text-white text-lg tracking-wide hidden md:block">CATerm</span>
        </div>
        <span class="text-[10px] px-1.5 py-0.5 rounded bg-sky-500/20 text-sky-400 font-mono hidden md:block">v2.0.4</span>
      </div>
      
      <nav class="p-2 space-y-1 text-sm font-medium">
        <a href="/" class="p-2.5 rounded-lg hover:bg-neutral-800/80 flex items-center gap-3 transition-colors text-neutral-200 hover:text-white">
          <svg class="w-5 h-5 shrink-0 text-sky-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6"></path></svg>
          <span class="hidden md:inline">Hosts</span>
        </a>
        <a href="/groups" class="p-2.5 rounded-lg hover:bg-neutral-800/80 flex items-center gap-3 transition-colors text-neutral-400 hover:text-white">
          <svg class="w-5 h-5 shrink-0 text-indigo-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"></path></svg>
          <span class="hidden md:inline">Groups</span>
        </a>
        <a href="/snippets" class="p-2.5 rounded-lg hover:bg-neutral-800/80 flex items-center gap-3 transition-colors text-neutral-400 hover:text-white">
          <svg class="w-5 h-5 shrink-0 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4"></path></svg>
          <span class="hidden md:inline">Snippets</span>
        </a>
        <a href="/port-forwarding" class="p-2.5 rounded-lg hover:bg-neutral-800/80 flex items-center gap-3 transition-colors text-neutral-400 hover:text-white">
          <svg class="w-5 h-5 shrink-0 text-amber-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4"></path></svg>
          <span class="hidden md:inline">Port forwarding</span>
        </a>
        <a href="/monitoring" class="p-2.5 rounded-lg hover:bg-neutral-800/80 flex items-center gap-3 transition-colors text-neutral-400 hover:text-white">
          <svg class="w-5 h-5 shrink-0 text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"></path></svg>
          <span class="hidden md:inline">Monitoring</span>
        </a>
        <a href="/sftp" class="p-2.5 rounded-lg hover:bg-neutral-800/80 flex items-center gap-3 transition-colors text-neutral-400 hover:text-white">
          <svg class="w-5 h-5 shrink-0 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"></path></svg>
          <span class="hidden md:inline">SFTP Files</span>
        </a>
        <a href="/ssh-keys" class="p-2.5 rounded-lg hover:bg-neutral-800/80 flex items-center gap-3 transition-colors text-neutral-400 hover:text-white">
          <svg class="w-5 h-5 shrink-0 text-rose-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z"></path></svg>
          <span class="hidden md:inline">SSH keys</span>
        </a>
      </nav>
    </div>

    <!-- Bottom Actions: Settings & User Identity / Lock -->
    <div class="p-2 space-y-1 border-t border-neutral-800/80">
      <a href="/settings" class="p-2.5 rounded-lg hover:bg-neutral-800/80 flex items-center gap-3 transition-colors text-neutral-400 hover:text-white">
        <svg class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"></path><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"></path></svg>
        <span class="hidden md:inline">Settings</span>
      </a>

      <!-- Profile & Lock Button -->
      <div class="p-2 rounded-lg bg-neutral-900/60 border border-neutral-800/50 flex items-center justify-between">
        <div class="hidden md:flex items-center gap-2 overflow-hidden">
          <div class="w-7 h-7 rounded-full bg-sky-500/20 text-sky-400 font-bold text-xs flex items-center justify-center shrink-0 border border-sky-500/30">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"></path></svg>
          </div>
          <div class="truncate text-xs">
            <p class="font-medium text-white truncate">Local Vault</p>
            <p class="text-[10px] text-emerald-400 truncate font-mono">Encrypted • Active</p>
          </div>
        </div>
        <button onclick={() => isUnlocked = false} title="Lock Vault" class="p-1.5 text-neutral-400 hover:text-amber-400 hover:bg-neutral-800 rounded transition-colors">
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"/></svg>
        </button>
      </div>
    </div>
  </aside>

  <!-- Main Content -->
  <main class="flex-1 flex flex-col h-screen overflow-hidden">
    <!-- Top Bar (Sessions Tab System Placeholder) -->
    <header class="h-12 border-b border-neutral-800 flex items-center px-4 bg-neutral-900 shrink-0 overflow-x-auto">
      <div class="flex gap-2 text-sm items-center">
        <a href="/" class="px-3 py-1 bg-neutral-800 text-white rounded-t-md border-t border-l border-r border-neutral-700 font-medium">Dashboard</a>
        {#each sessionTabs as tab (tab.id)}
          <div class="flex items-center gap-1 px-1 text-neutral-400 hover:text-neutral-200 hover:bg-neutral-800/50 rounded-t-md transition-colors">
            <a href="/session" class="py-1 pl-2">{tab.host.label}</a>
            <button
              onclick={() => closeTab(tab.id)}
              class="p-0.5 rounded hover:bg-neutral-700 hover:text-rose-400"
              title="Tutup sesi {tab.host.label}">
              <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-width="2" stroke-linecap="round" d="M6 18L18 6M6 6l12 12"></path></svg>
            </button>
          </div>
        {/each}
      </div>
    </header>
    
    <div class="flex-1 overflow-auto bg-[#0a0a0a] p-6 relative">
      {@render children()}
    </div>
  </main>
</div>
{/if}
