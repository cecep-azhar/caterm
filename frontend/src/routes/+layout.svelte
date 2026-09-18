<script lang="ts">
  import "../app.css";
  let { children } = $props();
</script>

<div class="flex h-screen bg-[#0e0e0e] text-neutral-300 font-sans">
  <!-- Sidebar -->
  <aside class="w-16 md:w-64 border-r border-neutral-800 flex flex-col justify-between">
    <div>
      <div class="h-12 border-b border-neutral-800 flex items-center justify-center md:justify-start px-4">
        <span class="font-bold text-white text-lg tracking-wide hidden md:block">CATerm</span>
        <span class="font-bold text-white text-lg block md:hidden">CT</span>
      </div>
      <nav class="mt-4 flex flex-col gap-1 px-2">
        <a href="/" class="p-2 rounded hover:bg-neutral-800 flex items-center gap-3 transition-colors">
          <svg class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6"></path></svg>
          <span class="hidden md:inline">Dashboard</span>
        </a>
        <a href="/hosts" class="p-2 rounded hover:bg-neutral-800 flex items-center gap-3 transition-colors">
          <svg class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01"></path></svg>
          <span class="hidden md:inline">Hosts</span>
        </a>
        <a href="/groups" class="p-2 rounded hover:bg-neutral-800 flex items-center gap-3 transition-colors">
          <svg class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"></path></svg>
          <span class="hidden md:inline">Groups</span>
        </a>
        <a href="/snippets" class="p-2 rounded hover:bg-neutral-800 flex items-center gap-3 transition-colors">
          <svg class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4"></path></svg>
          <span class="hidden md:inline">Snippets</span>
        </a>
      </nav>
    </div>
    <div class="p-2 mb-2">
      <a href="/settings" class="p-2 rounded hover:bg-neutral-800 flex items-center gap-3 transition-colors">
        <svg class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"></path><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"></path></svg>
        <span class="hidden md:inline">Settings</span>
      </a>
    </div>
  </aside>

  <!-- Main Content -->
  <main class="flex-1 flex flex-col h-screen overflow-hidden">
    <!-- Top Bar (Sessions Tab System Placeholder) -->
    <header class="h-12 border-b border-neutral-800 flex items-center px-4 bg-neutral-900 shrink-0 overflow-x-auto">
      <div class="flex gap-2 text-sm">
        <button class="px-3 py-1 bg-neutral-800 text-white rounded-t-md border-t border-l border-r border-neutral-700 font-medium">Dashboard <span class="text-neutral-500 ml-1 text-xs">⌘1</span></button>
        <button class="px-3 py-1 text-neutral-400 hover:text-neutral-200 hover:bg-neutral-800/50 rounded-t-md transition-colors">
          <a href="/session" class="w-full h-full block">Session 1 <span class="text-neutral-600 ml-1 text-xs">⌘2</span></a>
        </button>
        <button class="px-3 py-1 text-neutral-400 hover:text-neutral-200 hover:bg-neutral-800/50 rounded-t-md transition-colors">Session 2 <span class="text-neutral-600 ml-1 text-xs">⌘3</span></button>
      </div>
    </header>
    
    <div class="flex-1 overflow-auto bg-[#0a0a0a] p-6 relative">
      {@render children()}
    </div>
  </main>
</div>