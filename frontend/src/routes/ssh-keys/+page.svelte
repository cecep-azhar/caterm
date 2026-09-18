<script lang="ts">
</script>

<div class="max-w-4xl mx-auto space-y-6">
  <div class="flex items-center gap-3">
    <div class="p-2 bg-rose-500/10 text-rose-400 rounded-lg border border-rose-500/20">
      <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z"></path></svg>
    </div>
    <div>
      <h1 class="text-2xl font-bold text-white">SSH Keys</h1>
      <p class="text-sm text-neutral-400">Manage, generate, and import SSH keypairs (RSA / ED25519) securely stored in the local vault.</p>
    </div>
  </div>

  <div class="p-8 border border-neutral-800 rounded-xl bg-neutral-900/30 text-center flex flex-col items-center justify-center">
    <svg class="w-12 h-12 text-neutral-600 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"></path></svg>
    <h3 class="text-lg font-medium text-white mb-2">No keys in vault</h3>
    <p class="text-sm text-neutral-400 max-w-md">Generate a new ED25519 keypair or import an existing private key to authenticate securely without passwords.</p>
    <div class="mt-6 flex gap-3">
      <button class="px-4 py-2 bg-neutral-800 hover:bg-neutral-700 text-white rounded-lg text-sm font-medium transition-colors border border-neutral-700">
        Import Key
      </button>
      <button class="px-4 py-2 bg-sky-600 hover:bg-sky-500 text-white rounded-lg text-sm font-medium transition-colors shadow-lg shadow-sky-600/20">
        Generate Keypair
      </button>
    </div>
  </div>
</div>