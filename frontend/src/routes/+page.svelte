<script lang="ts">
  let isReportModalOpen = $state(false);
</script>

<div class="max-w-4xl mx-auto space-y-8">
  <div>
    <h1 class="text-3xl font-bold text-white tracking-tight">Welcome to CATerm</h1>
    <p class="text-neutral-400 mt-2">Zero-Knowledge, Local-First, High-Performance Terminal & Host Manager.</p>
  </div>

  <!-- Feature Grid -->
  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
    <div class="p-5 bg-neutral-900 border border-neutral-800 rounded-lg">
      <div class="w-10 h-10 bg-indigo-500/10 text-indigo-400 rounded-lg flex items-center justify-center mb-3">
        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"></path></svg>
      </div>
      <h2 class="text-lg font-semibold text-white">E2EE Storage</h2>
      <p class="text-neutral-400 text-sm mt-1">Zero-Knowledge local encryption. Your host keys and credentials never leave your system unencrypted.</p>
    </div>

    <div class="p-5 bg-neutral-900 border border-neutral-800 rounded-lg">
      <div class="w-10 h-10 bg-emerald-500/10 text-emerald-400 rounded-lg flex items-center justify-center mb-3">
        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4"></path></svg>
      </div>
      <h2 class="text-lg font-semibold text-white">SFTP Browser</h2>
      <p class="text-neutral-400 text-sm mt-1">Built-in high speed SFTP file transfers alongside active SSH split terminals.</p>
    </div>

    <div class="p-5 bg-neutral-900 border border-neutral-800 rounded-lg">
      <div class="w-10 h-10 bg-sky-500/10 text-sky-400 rounded-lg flex items-center justify-center mb-3">
        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2"></path></svg>
      </div>
      <h2 class="text-lg font-semibold text-white">Audit Log</h2>
      <p class="text-neutral-400 text-sm mt-1">Local audit logging for commands and security operations for auditability.</p>
    </div>

    <div class="p-5 bg-neutral-900 border border-neutral-800 rounded-lg">
      <div class="w-10 h-10 bg-purple-500/10 text-purple-400 rounded-lg flex items-center justify-center mb-3">
        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z"></path></svg>
      </div>
      <h2 class="text-lg font-semibold text-white">AI Terminal Assistant</h2>
      <p class="text-neutral-400 text-sm mt-1">Integrated local & cloud AI command helper for command generation.</p>
    </div>

    <div class="p-5 bg-neutral-900 border border-neutral-800 rounded-lg md:col-span-2">
      <div class="w-10 h-10 bg-amber-500/10 text-amber-400 rounded-lg flex items-center justify-center mb-3">
        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z"></path></svg>
      </div>
      <h2 class="text-lg font-semibold text-white">Dedikasi</h2>
      <p class="text-neutral-400 text-sm mt-1">Didedikasikan untuk putraku, <strong>Umar Abdillah</strong>, dan pengembangan perangkat lunak open-source lokal yang kuat, aman, dan tanpa biaya artifisial. Dibuat oleh <strong>Cecep Azhar</strong> (Fathforce).</p>
    </div>
  </div>

  <!-- Action Bar -->
  <div class="pt-4 border-t border-neutral-800 flex justify-between items-center">
    <button 
      onclick={() => isReportModalOpen = true}
      class="px-4 py-2 bg-neutral-800 hover:bg-neutral-700 text-neutral-200 text-sm font-medium rounded-md transition-colors flex items-center gap-2">
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"></path></svg>
      Report Issue
    </button>
  </div>
</div>

<!-- Report Issue Modal -->
{#if isReportModalOpen}
  <div class="fixed inset-0 bg-black/70 flex items-center justify-center p-4 z-50">
    <div class="bg-neutral-900 border border-neutral-800 rounded-lg p-6 max-w-md w-full space-y-4">
      <h3 class="text-xl font-bold text-white">Report an Issue</h3>
      <p class="text-neutral-400 text-sm">Send bug reports or suggestions directly to the author email below:</p>
      <div class="p-3 bg-neutral-950 border border-neutral-800 rounded font-mono text-sm text-sky-400">
        cecep.azhtech@gmail.com
      </div>
      <div class="flex justify-end gap-2 pt-2">
        <button 
          onclick={() => isReportModalOpen = false} 
          class="px-4 py-2 bg-neutral-800 hover:bg-neutral-700 text-neutral-300 rounded text-sm font-medium">
          Close
        </button>
        <a 
          href="mailto:cecep.azhtech@gmail.com?subject=CATerm%20Issue%20Report" 
          class="px-4 py-2 bg-sky-600 hover:bg-sky-500 text-white rounded text-sm font-medium">
          Open Mail App
        </a>
      </div>
    </div>
  </div>
{/if}