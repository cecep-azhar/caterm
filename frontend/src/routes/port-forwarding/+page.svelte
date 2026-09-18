<script lang="ts">
</script>

<div class="max-w-4xl mx-auto space-y-6">
  <div class="flex items-center gap-3">
    <div class="p-2 bg-amber-500/10 text-amber-400 rounded-lg border border-amber-500/20">
      <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4"></path></svg>
    </div>
    <div>
      <h1 class="text-2xl font-bold text-white">Port Forwarding</h1>
      <p class="text-sm text-neutral-400">Manage Local, Remote, and Dynamic SSH tunnels.</p>
    </div>
  </div>
  
  <div class="p-8 border border-neutral-800 rounded-xl bg-neutral-900/30 text-center flex flex-col items-center justify-center">
    <svg class="w-12 h-12 text-neutral-600 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"></path></svg>
    <h3 class="text-lg font-medium text-white mb-2">No active tunnels</h3>
    <p class="text-sm text-neutral-400 max-w-md">Create port forwarding rules to securely tunnel local ports through your connected SSH hosts.</p>
    <button class="mt-6 px-4 py-2 bg-sky-600 hover:bg-sky-500 text-white rounded-lg text-sm font-medium transition-colors shadow-lg shadow-sky-600/20">
      Add Port Forwarding Rule
    </button>
  </div>
</div>