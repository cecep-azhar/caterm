<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import TotpBadge from '$lib/components/TotpBadge.svelte';
  import { showToast, confirmModal } from '$lib/stores/uiNotifications.svelte';
  import { t } from '$lib/i18n/index.svelte';

  interface TotpEntry {
    id: string;
    label: string;
    issuer?: string;
    hasSecret: boolean;
    token: string;
    remainingSeconds: number;
    createdAt: number;
    updatedAt: number;
  }

  let entries = $state<TotpEntry[]>([]);
  let searchQuery = $state('');
  let isAddModalOpen = $state(false);
  let isLoading = $state(false);
  let timerInterval: any = null;

  // Form State
  let editingId = $state<string | null>(null);
  let formLabel = $state('');
  let formIssuer = $state('');
  let formSecret = $state('');
  let formError = $state('');

  const filteredEntries = $derived.by(() => {
    const q = searchQuery.toLowerCase().trim();
    if (!q) return entries;
    return entries.filter(
      (e) =>
        e.label.toLowerCase().includes(q) ||
        (e.issuer && e.issuer.toLowerCase().includes(q))
    );
  });

  onMount(() => {
    refreshEntries();
    // Live countdown update every second
    timerInterval = setInterval(() => {
      let needsFetch = false;
      entries = entries.map((entry) => {
        if (entry.remainingSeconds <= 1) {
          needsFetch = true;
          return { ...entry, remainingSeconds: 30 };
        }
        return { ...entry, remainingSeconds: entry.remainingSeconds - 1 };
      });
      if (needsFetch) {
        refreshEntries();
      }
    }, 1000);
  });

  onDestroy(() => {
    if (timerInterval) clearInterval(timerInterval);
  });

  async function refreshEntries() {
    try {
      entries = await invoke('list_totp_entries');
    } catch (e: any) {
      console.warn('Failed to load 2FA entries:', e);
    }
  }

  function openAddModal() {
    editingId = null;
    formLabel = '';
    formIssuer = '';
    formSecret = '';
    formError = '';
    isAddModalOpen = true;
  }

  function openEditModal(entry: TotpEntry) {
    editingId = entry.id;
    formLabel = entry.label;
    formIssuer = entry.issuer || '';
    formSecret = '';
    formError = '';
    isAddModalOpen = true;
  }

  async function handleSave() {
    if (!formLabel.trim()) {
      formError = 'Label token 2FA wajib diisi';
      return;
    }
    if (!editingId && !formSecret.trim()) {
      formError = 'Secret key Base32 atau URL otpauth:// wajib diisi';
      return;
    }

    isLoading = true;
    formError = '';
    try {
      await invoke('save_totp_entry', {
        input: {
          id: editingId ?? undefined,
          label: formLabel.trim(),
          issuer: formIssuer.trim() || undefined,
          secret: formSecret.trim() || undefined
        }
      });
      isAddModalOpen = false;
      showToast(editingId ? 'Token 2FA diperbarui!' : 'Token 2FA berhasil ditambahkan!', 'success');
      await refreshEntries();
    } catch (err: any) {
      formError = typeof err === 'string' ? err : err?.message || 'Gagal menyimpan 2FA token';
      showToast(formError, 'error');
    } finally {
      isLoading = false;
    }
  }

  async function handleDelete(id: string, label: string) {
    const confirmed = await confirmModal(
      `Apakah Anda yakin ingin menghapus token 2FA "${label}"? Tindakan ini tidak dapat dibatalkan.`,
      'Hapus Token 2FA',
      true,
      'Hapus',
      'Batal'
    );
    if (confirmed) {
      try {
        await invoke('delete_totp_entry', { id });
        showToast(`Token "${label}" dihapus!`, 'success');
        await refreshEntries();
      } catch (e: any) {
        showToast(String(e), 'error');
      }
    }
  }
</script>

<svelte:head>
  <title>2FA Authenticator Vault · CATerm</title>
</svelte:head>

<div class="space-y-6 max-w-5xl mx-auto">
  <!-- Header -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-neutral-200 dark:border-neutral-800 pb-5">
    <div>
      <div class="flex items-center gap-2.5">
        <div class="w-8 h-8 rounded-lg bg-sky-500/10 text-sky-600 dark:text-sky-400 flex items-center justify-center border border-sky-500/20">
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
          </svg>
        </div>
        <h1 class="text-xl font-bold text-neutral-900 dark:text-white">2FA Authenticator Vault</h1>
      </div>
      <p class="text-xs text-neutral-500 dark:text-neutral-400 mt-1">
        Kelola kode verifikasi 2FA (RFC 6238 TOTP) terenkripsi zero-knowledge dengan fitur auto-inject SSH.
      </p>
    </div>

    <button
      type="button"
      onclick={openAddModal}
      class="inline-flex items-center justify-center gap-2 px-4 py-2 rounded-xl text-xs font-semibold bg-sky-600 hover:bg-sky-500 text-white shadow-md shadow-sky-600/20 transition-all shrink-0"
    >
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
      </svg>
      Tambah Token 2FA
    </button>
  </div>

  <!-- Search & Stats Bar -->
  <div class="flex items-center justify-between gap-3">
    <div class="relative flex-1 max-w-md">
      <input
        type="text"
        bind:value={searchQuery}
        placeholder="Cari token 2FA, server, atau akun..."
        class="w-full pl-9 pr-3 py-2 text-xs rounded-xl bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 text-neutral-900 dark:text-white placeholder-neutral-400 focus:outline-none focus:border-sky-500 shadow-2xs"
      />
      <svg class="w-4 h-4 text-neutral-400 absolute left-3 top-2.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
      </svg>
    </div>
    <span class="text-xs font-medium text-neutral-500 shrink-0">
      {filteredEntries.length} Token Tersimpan
    </span>
  </div>

  <!-- Cards Grid -->
  {#if filteredEntries.length === 0}
    <div class="text-center py-16 px-4 bg-white dark:bg-neutral-900/40 rounded-2xl border border-neutral-200 dark:border-neutral-800 space-y-3">
      <div class="w-12 h-12 rounded-full bg-sky-500/10 text-sky-500 flex items-center justify-center mx-auto border border-sky-500/20">
        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
        </svg>
      </div>
      <h3 class="text-sm font-semibold text-neutral-900 dark:text-white">Belum Ada Token 2FA</h3>
      <p class="text-xs text-neutral-500 max-w-sm mx-auto">
        Tambahkan secret key 2FA server produksi, jump host, AWS, Cloudflare, atau GitHub Anda di sini untuk kemudahan 1-klik copy & auto-inject SSH.
      </p>
      <button
        type="button"
        onclick={openAddModal}
        class="mt-2 inline-flex items-center gap-1.5 px-3.5 py-1.5 rounded-lg text-xs font-semibold bg-neutral-100 dark:bg-neutral-800 hover:bg-sky-500 hover:text-white text-neutral-700 dark:text-neutral-300 transition-colors"
      >
        + Tambah Token Pertama
      </button>
    </div>
  {:else}
    <div class="grid grid-cols-1 md:grid-cols-2 gap-3.5">
      {#each filteredEntries as entry (entry.id)}
        <div class="relative group">
          <TotpBadge
            token={entry.token}
            remainingSeconds={entry.remainingSeconds}
            label={entry.label}
            issuer={entry.issuer}
          />
          <!-- Action buttons on hover -->
          <div class="absolute right-14 top-3 flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
            <button
              type="button"
              onclick={() => openEditModal(entry)}
              class="p-1.5 rounded-md bg-neutral-100 dark:bg-neutral-800 hover:text-sky-500 text-neutral-500 transition-colors"
              title="Edit Token"
              aria-label="Edit Token"
            >
              <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
              </svg>
            </button>
            <button
              type="button"
              onclick={() => handleDelete(entry.id, entry.label)}
              class="p-1.5 rounded-md bg-neutral-100 dark:bg-neutral-800 hover:text-rose-500 text-neutral-500 transition-colors"
              title="Hapus Token"
              aria-label="Hapus Token"
            >
              <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
              </svg>
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<!-- Add / Edit 2FA Modal -->
{#if isAddModalOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 dark:bg-black/80 backdrop-blur-xs">
    <div
      role="dialog"
      aria-modal="true"
      class="w-full max-w-md bg-white dark:bg-[#141414] border border-neutral-200 dark:border-neutral-800 rounded-2xl p-6 space-y-4 shadow-2xl animate-in fade-in zoom-in-95 duration-150"
    >
      <div class="flex items-center justify-between border-b border-neutral-200 dark:border-neutral-800 pb-3">
        <h3 class="text-base font-bold text-neutral-900 dark:text-white">
          {editingId ? 'Edit Token 2FA' : 'Tambah Token 2FA Baru'}
        </h3>
        <button
          type="button"
          onclick={() => (isAddModalOpen = false)}
          class="text-neutral-400 hover:text-neutral-600 dark:hover:text-neutral-200 text-lg font-bold"
        >
          ×
        </button>
      </div>

      {#if formError}
        <div class="p-2.5 text-xs rounded-lg bg-rose-500/10 border border-rose-500/30 text-rose-600 dark:text-rose-400">
          {formError}
        </div>
      {/if}

      <div class="space-y-3 text-xs">
        <div>
          <label for="totp-label" class="block font-medium text-neutral-700 dark:text-neutral-300 mb-1">
            Nama Akun / Server *
          </label>
          <input
            id="totp-label"
            type="text"
            bind:value={formLabel}
            placeholder="misal: Production Bastion / AWS Root"
            class="w-full px-3 py-2 rounded-lg bg-neutral-50 dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500"
          />
        </div>

        <div>
          <label for="totp-issuer" class="block font-medium text-neutral-700 dark:text-neutral-300 mb-1">
            Penerbit / Layanan (Opsional)
          </label>
          <input
            id="totp-issuer"
            type="text"
            bind:value={formIssuer}
            placeholder="misal: AWS, Cloudflare, GitHub, Gerlink"
            class="w-full px-3 py-2 rounded-lg bg-neutral-50 dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500"
          />
        </div>

        <div>
          <label for="totp-secret" class="block font-medium text-neutral-700 dark:text-neutral-300 mb-1">
            Secret Key (Base32 atau URL otpauth://) {editingId ? '(Kosongkan jika tidak ingin mengubah)' : '*'}
          </label>
          <input
            id="totp-secret"
            type="password"
            bind:value={formSecret}
            placeholder="misal: JBSWY3DPEHPK3PXP atau otpauth://totp/..."
            class="w-full px-3 py-2 rounded-lg font-mono bg-neutral-50 dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500"
          />
          <p class="text-[11px] text-neutral-500 mt-1">
            Kunci rahasia akan langsung dienkripsi menggunakan Zero-Knowledge AES-256-GCM vault.
          </p>
        </div>
      </div>

      <div class="flex justify-end gap-2.5 pt-4 border-t border-neutral-200 dark:border-neutral-800">
        <button
          type="button"
          onclick={() => (isAddModalOpen = false)}
          class="px-4 py-2 rounded-lg text-xs font-semibold bg-neutral-100 dark:bg-neutral-800 hover:bg-neutral-200 dark:hover:bg-neutral-700 text-neutral-700 dark:text-neutral-300 transition-colors"
        >
          Batal
        </button>
        <button
          type="button"
          onclick={handleSave}
          disabled={isLoading}
          class="px-4 py-2 rounded-lg text-xs font-semibold bg-sky-600 hover:bg-sky-500 disabled:opacity-50 text-white shadow-md shadow-sky-600/20 transition-colors"
        >
          {isLoading ? 'Menyimpan...' : 'Simpan Token'}
        </button>
      </div>
    </div>
  </div>
{/if}
