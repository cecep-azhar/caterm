<script lang="ts">
</script>

<div class="max-w-4xl mx-auto space-y-6">
  <div class="flex items-center gap-3">
    <div class="p-2 bg-purple-500/10 text-purple-400 rounded-lg border border-purple-500/20">
      <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"></path></svg>
    </div>
    <div>
      <h1 class="text-2xl font-bold text-white">System Monitoring</h1>
      <p class="text-sm text-neutral-400">Real-time resource utilization (CPU, RAM, Disk, Network) across connected hosts.</p>
    </div>
  </div>

  <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
    <div class="p-6 border border-neutral-800 rounded-xl bg-neutral-900/40">
      <div class="flex items-center justify-between mb-4">
        <span class="font-medium text-white">YPC Server</span>
        <span class="px-2 py-0.5 rounded text-xs bg-emerald-500/20 text-emerald-400">Online</span>
      </div>
      <div class="space-y-3 text-sm">
        <div>
          <div class="flex justify-between text-xs text-neutral-400 mb-1">
            <span>CPU Usage</span>
            <span>14%</span>
          </div>
          <div class="w-full h-2 bg-neutral-800 rounded-full overflow-hidden">
            <div class="w-[14%] h-full bg-sky-500 rounded-full"></div>
          </div>
        </div>
        <div>
          <div class="flex justify-between text-xs text-neutral-400 mb-1">
            <span>RAM Usage</span>
            <span>4.2 GB / 16 GB</span>
          </div>
          <div class="w-full h-2 bg-neutral-800 rounded-full overflow-hidden">
            <div class="w-[26%] h-full bg-indigo-500 rounded-full"></div>
          </div>
        </div>
      </div>
    </div>

    <div class="p-6 border border-neutral-800 rounded-xl bg-neutral-900/40">
      <div class="flex items-center justify-between mb-4">
        <span class="font-medium text-white">XPC Server</span>
        <span class="px-2 py-0.5 rounded text-xs bg-emerald-500/20 text-emerald-400">Online</span>
      </div>
      <div class="space-y-3 text-sm">
        <div>
          <div class="flex justify-between text-xs text-neutral-400 mb-1">
            <span>CPU Usage</span>
            <span>8%</span>
          </div>
          <div class="w-full h-2 bg-neutral-800 rounded-full overflow-hidden">
            <div class="w-[8%] h-full bg-sky-500 rounded-full"></div>
          </div>
        </div>
        <div>
          <div class="flex justify-between text-xs text-neutral-400 mb-1">
            <span>RAM Usage</span>
            <span>6.1 GB / 32 GB</span>
          </div>
          <div class="w-full h-2 bg-neutral-800 rounded-full overflow-hidden">
            <div class="w-[19%] h-full bg-indigo-500 rounded-full"></div>
          </div>
        </div>
      </div>
    </div>
  </div>
</div>