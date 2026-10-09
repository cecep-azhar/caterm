<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import {
    runNetworkSecurityAudit,
    runThreatWatchdog,
    runTlsAudit,
    type AuditReport,
    type ThreatWatchdogResult,
    type TlsAuditResult
  } from '$lib/api/audit';
  import { listHosts, type HostRecord } from '$lib/api/hosts';
  import { saveInvestigation } from '$lib/api/investigations';
  import { showToast } from '$lib/stores/uiNotifications.svelte';
  import { errorText } from '$lib/errors';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import ProGate from '$lib/components/ProGate.svelte';

  let hosts = $state<HostRecord[]>([]);
  let selectedHostId = $state<string>('');
  let activeTab = $state<'exposure' | 'hardening' | 'watchdog' | 'tls'>('exposure');

  // Loading States
  let isRunningSecurity = $state(false);
  let isRunningWatchdog = $state(false);
  let isRunningTls = $state(false);
  let isSavingIncident = $state(false);

  // Results State
  let securityReport = $state<AuditReport | null>(null);
  let watchdogResult = $state<ThreatWatchdogResult | null>(null);
  let tlsResult = $state<TlsAuditResult | null>(null);

  // TLS Inputs
  let tlsHost = $state('cloudflare.com');
  let tlsPort = $state<number>(443);

  onMount(async () => {
    try {
      hosts = await listHosts();
      const qHost = page.url.searchParams.get('host');
      if (qHost) {
        selectedHostId = qHost;
      } else if (hosts.length > 0) {
        selectedHostId = hosts[0].id;
      } else {
        selectedHostId = 'local';
      }
    } catch {
      selectedHostId = 'local';
    }
  });

  async function startSecurityAudit() {
    if (!selectedHostId) {
      showToast('Pilih host target terlebih dahulu', 'error');
      return;
    }
    isRunningSecurity = true;
    securityReport = null;
    try {
      const res = await runNetworkSecurityAudit(selectedHostId);
      securityReport = res;
      showToast(`Audit keamanan selesai! Skor: ${res.summaryScore}%`, res.summaryScore > 70 ? 'success' : 'error');
    } catch (e) {
      showToast(`Audit gagal: ${errorText(e)}`, 'error');
    } finally {
      isRunningSecurity = false;
    }
  }

  async function startThreatWatchdog() {
    if (!selectedHostId) return;
    isRunningWatchdog = true;
    try {
      watchdogResult = await runThreatWatchdog(selectedHostId);
      showToast(`Threat Watchdog selesai: Status ${watchdogResult.threatLevel}`, watchdogResult.threatLevel === 'CLEAN' ? 'success' : 'error');
    } catch (e) {
      showToast(`Watchdog gagal: ${errorText(e)}`, 'error');
    } finally {
      isRunningWatchdog = false;
    }
  }

  async function startTlsAudit() {
    if (!tlsHost.trim()) return;
    isRunningTls = true;
    try {
      tlsResult = await runTlsAudit(tlsHost, tlsPort);
      showToast(`TLS Audit selesai: ${tlsResult.status}`, tlsResult.status === 'HEALTHY' ? 'success' : 'error');
    } catch (e) {
      showToast(`TLS Audit gagal: ${errorText(e)}`, 'error');
    } finally {
      isRunningTls = false;
    }
  }

  async function exportToInvestigation() {
    if (!securityReport) return;
    isSavingIncident = true;
    try {
      const hostLabel = getHostLabel(securityReport.hostId);
      const title = `Security Incident & Audit Report - Host ${hostLabel} (Skor: ${securityReport.summaryScore}%)`;
      const desc = `Laporan audit keamanan otomatis CATerm Security Suite.
Skor Keamanan: ${securityReport.summaryScore}%
Temuan Inbound Wildcard: ${securityReport.inboundPorts.filter((p) => p.riskLevel === 'CRITICAL' || p.riskLevel === 'HIGH').length} port
Temuan Egress Leak: ${securityReport.egressResults.filter((e) => e.status === 'OPEN_LEAK').length} port
SSH Hardening Non-Pass: ${securityReport.hardeningChecklist.filter((h) => h.status !== 'PASS').length} check`;

      await saveInvestigation({
        title,
        description: desc,
        host_id: securityReport.hostId === 'local' ? '' : securityReport.hostId,
        status: securityReport.summaryScore < 70 ? 'OPEN' : 'CLOSED',
        notes: JSON.stringify(
          {
            security: securityReport,
            watchdog: watchdogResult,
            tls: tlsResult,
            exportedAt: new Date().toISOString()
          },
          null,
          2
        )
      });
      showToast('Laporan audit berhasil diekspor ke menu Investigations!', 'success');
    } catch (e) {
      showToast(`Gagal menyimpan incident: ${errorText(e)}`, 'error');
    } finally {
      isSavingIncident = false;
    }
  }

  function getHostLabel(id: string): string {
    if (id === 'local') return 'Local Machine (Fedora)';
    const found = hosts.find((h) => h.id === id);
    return found ? `${found.label} (${found.address})` : id;
  }
</script>

<div class="p-6 max-w-7xl mx-auto space-y-6">
  <!-- HEADER -->
  <div class="flex flex-col md:flex-row md:items-center justify-between gap-4">
    <div class="flex items-center gap-3">
      <div class="w-10 h-10 rounded-xl bg-neutral-900 border border-neutral-800 flex items-center justify-center text-emerald-500 shrink-0">
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"/></svg>
      </div>
      <div>
        <h1 class="text-xl font-bold text-white tracking-tight">Security</h1>
        <p class="text-xs text-neutral-400 mt-0.5">Audit celah port listening 0.0.0.0, outbound hole hunter, integritas SSH hardening, dan live reverse shell watchdog.</p>
      </div>
    </div>
    <div class="flex flex-wrap items-center gap-3">
      <!-- Host Selector -->
      <div class="flex items-center gap-2 bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-xl px-3 py-1.5 shadow-xs">
        <label for="host-select" class="text-xs font-semibold text-neutral-500 uppercase">Target Host:</label>
        <select
          id="host-select"
          bind:value={selectedHostId}
          class="bg-transparent text-xs font-bold text-neutral-900 dark:text-white focus:outline-none cursor-pointer"
        >
          <option value="local">Localhost (Mesin Ini)</option>
          {#each hosts as h}
            <option value={h.id}>{h.label} ({h.address})</option>
          {/each}
        </select>
      </div>

      <!-- Quick Run Button -->
      <button
        onclick={startSecurityAudit}
        disabled={isRunningSecurity || !selectedHostId}
        class="px-4 py-2 bg-emerald-600 hover:bg-emerald-500 disabled:opacity-50 text-white text-xs font-semibold rounded-xl shadow-md shadow-emerald-600/20 flex items-center gap-2 transition"
      >
        {#if isRunningSecurity}
          <span class="animate-spin text-xs">⏳</span>
          <span>Scanning...</span>
        {:else}
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"/></svg>
          <span>Run Security Audit</span>
        {/if}
      </button>

      {#if securityReport}
        <button
          onclick={exportToInvestigation}
          disabled={isSavingIncident}
          class="px-3.5 py-2 bg-neutral-100 hover:bg-neutral-200 dark:bg-neutral-800 dark:hover:bg-neutral-700 text-neutral-700 dark:text-neutral-200 text-xs font-semibold rounded-xl border border-neutral-300 dark:border-neutral-700 flex items-center gap-1.5 transition"
        >
          <span>📁 {isSavingIncident ? 'Mengekspor...' : 'Export Incident'}</span>
        </button>
      {/if}
    </div>
  </div>

  <!-- NAVIGATION TABS -->
  <div class="flex items-center gap-2 border-b border-neutral-200 dark:border-neutral-800 pb-2 overflow-x-auto text-xs font-semibold">
    <button
      onclick={() => (activeTab = 'exposure')}
      class="px-4 py-2 rounded-xl transition flex items-center gap-2 whitespace-nowrap {activeTab === 'exposure' ? 'bg-white dark:bg-neutral-800 text-emerald-600 dark:text-emerald-400 shadow-sm border border-neutral-200/50 dark:border-neutral-700' : 'text-neutral-600 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white'}"
    >
      <span>🛡️ Exposure & Hole Hunter</span>
    </button>
    <button
      onclick={() => (activeTab = 'hardening')}
      class="px-4 py-2 rounded-xl transition flex items-center gap-2 whitespace-nowrap {activeTab === 'hardening' ? 'bg-white dark:bg-neutral-800 text-sky-600 dark:text-sky-400 shadow-sm border border-neutral-200/50 dark:border-neutral-700' : 'text-neutral-600 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white'}"
    >
      <span>🔒 SSH Hardening Checklist</span>
    </button>
    <button
      onclick={() => (activeTab = 'watchdog')}
      class="px-4 py-2 rounded-xl transition flex items-center gap-2 whitespace-nowrap {activeTab === 'watchdog' ? 'bg-white dark:bg-neutral-800 text-rose-600 dark:text-rose-400 shadow-sm border border-neutral-200/50 dark:border-neutral-700' : 'text-neutral-600 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white'}"
    >
      <span>🚨 Threat Watchdog (Pro)</span>
      <span class="text-[9px] px-1.5 py-0.5 rounded bg-rose-500/20 text-rose-500 uppercase font-black">PRO</span>
    </button>
    <button
      onclick={() => (activeTab = 'tls')}
      class="px-4 py-2 rounded-xl transition flex items-center gap-2 whitespace-nowrap {activeTab === 'tls' ? 'bg-white dark:bg-neutral-800 text-amber-600 dark:text-amber-400 shadow-sm border border-neutral-200/50 dark:border-neutral-700' : 'text-neutral-600 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white'}"
    >
      <span>📜 TLS & SSL Auditor</span>
    </button>
  </div>

  <!-- TAB 1: EXPOSURE & HOLE HUNTER -->
  {#if activeTab === 'exposure'}
    <div class="space-y-6">
      {#if securityReport}
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
          <div class="p-4 rounded-2xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900/60 shadow-xs">
            <span class="text-xs font-semibold text-neutral-500 uppercase">Skor Keamanan</span>
            <div class="flex items-baseline gap-2 mt-2">
              <span class="text-3xl font-black {securityReport.summaryScore >= 80 ? 'text-emerald-500' : securityReport.summaryScore >= 60 ? 'text-amber-500' : 'text-rose-500'}">
                {securityReport.summaryScore}%
              </span>
            </div>
          </div>
          <div class="p-4 rounded-2xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900/60 shadow-xs">
            <span class="text-xs font-semibold text-neutral-500 uppercase">Egress Leaks</span>
            <div class="text-2xl font-bold mt-2 {securityReport.egressResults.filter((e) => e.status === 'OPEN_LEAK').length > 0 ? 'text-rose-500' : 'text-emerald-500'}">
              {securityReport.egressResults.filter((e) => e.status === 'OPEN_LEAK').length} port
            </div>
          </div>
          <div class="p-4 rounded-2xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900/60 shadow-xs">
            <span class="text-xs font-semibold text-neutral-500 uppercase">Inbound Wildcard (0.0.0.0)</span>
            <div class="text-2xl font-bold mt-2 {securityReport.inboundPorts.filter((p) => p.isWildcard && (p.riskLevel === 'CRITICAL' || p.riskLevel === 'HIGH')).length > 0 ? 'text-amber-500' : 'text-emerald-500'}">
              {securityReport.inboundPorts.filter((p) => p.isWildcard && (p.riskLevel === 'CRITICAL' || p.riskLevel === 'HIGH')).length} port
            </div>
          </div>
          <div class="p-4 rounded-2xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900/60 shadow-xs">
            <span class="text-xs font-semibold text-neutral-500 uppercase">Latency Ping Target</span>
            <div class="text-2xl font-bold text-sky-500 mt-2">
              {securityReport.latencyProbe.avgLatencyMs.toFixed(1)} ms
            </div>
          </div>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div class="border border-neutral-200 dark:border-neutral-800 rounded-2xl p-5 bg-white dark:bg-neutral-900/60 space-y-3">
            <h4 class="text-xs font-bold uppercase tracking-wider text-neutral-500">Inbound Exposure ({securityReport.inboundPorts.length})</h4>
            <div class="max-h-72 overflow-y-auto space-y-1.5 text-xs font-mono">
              {#each securityReport.inboundPorts as p}
                <div class="p-2.5 rounded-xl bg-neutral-50 dark:bg-neutral-800/50 flex items-center justify-between border border-neutral-100 dark:border-neutral-800/80">
                  <div>
                    <span class="font-bold text-neutral-900 dark:text-white">{p.localAddress}</span>
                    <span class="text-neutral-400 text-[11px] block">{p.process}</span>
                  </div>
                  <span class="text-[10px] px-2 py-0.5 rounded font-bold {p.riskLevel === 'CRITICAL' ? 'bg-rose-500/20 text-rose-500' : p.riskLevel === 'HIGH' ? 'bg-amber-500/20 text-amber-500' : 'bg-neutral-200 dark:bg-neutral-700 text-neutral-400'}">
                    {p.riskLevel}
                  </span>
                </div>
              {/each}
            </div>
          </div>

          <div class="border border-neutral-200 dark:border-neutral-800 rounded-2xl p-5 bg-white dark:bg-neutral-900/60 space-y-3">
            <h4 class="text-xs font-bold uppercase tracking-wider text-neutral-500">Egress Hole Hunter ({securityReport.egressResults.length})</h4>
            <div class="max-h-72 overflow-y-auto space-y-1.5 text-xs font-mono">
              {#each securityReport.egressResults as e}
                <div class="p-2.5 rounded-xl bg-neutral-50 dark:bg-neutral-800/50 flex items-center justify-between border border-neutral-100 dark:border-neutral-800/80">
                  <div>
                    <span class="font-bold text-neutral-900 dark:text-white">{e.port} ({e.serviceLabel})</span>
                    <span class="text-neutral-400 text-[11px] block">{e.target}</span>
                  </div>
                  <span class="text-[10px] px-2 py-0.5 rounded font-bold {e.status === 'OPEN_LEAK' ? 'bg-rose-500/20 text-rose-500' : 'bg-emerald-500/20 text-emerald-500'}">
                    {e.status}
                  </span>
                </div>
              {/each}
            </div>
          </div>
        </div>
      {:else}
        <div class="py-16 text-center text-neutral-400 text-xs bg-white dark:bg-neutral-900/40 rounded-2xl border border-neutral-200 dark:border-neutral-800">
          Tekan "Run Security Audit" untuk memindai exposure port dan lubang outbound pada target host.
        </div>
      {/if}
    </div>
  {/if}

  <!-- TAB 2: SSH HARDENING -->
  {#if activeTab === 'hardening'}
    <div class="space-y-4">
      {#if securityReport}
        <div class="border border-neutral-200 dark:border-neutral-800 rounded-2xl p-5 bg-white dark:bg-neutral-900/60 space-y-3">
          <h4 class="text-xs font-bold uppercase tracking-wider text-neutral-500">SSH Configuration Hardening Audit</h4>
          <div class="space-y-2 text-xs">
            {#each securityReport.hardeningChecklist as h}
              <div class="p-3 rounded-xl bg-neutral-50 dark:bg-neutral-800/50 flex items-center justify-between border border-neutral-100 dark:border-neutral-800/80">
                <div class="space-y-0.5">
                  <span class="font-bold text-neutral-900 dark:text-white">{h.checkName}</span>
                  <p class="text-neutral-500 text-[11px]">{h.details}</p>
                </div>
                <span class="text-[10px] px-2 py-0.5 rounded font-bold {h.status === 'PASS' ? 'bg-emerald-500/20 text-emerald-500' : h.status === 'FAIL' ? 'bg-rose-500/20 text-rose-500' : 'bg-amber-500/20 text-amber-500'}">
                  {h.status}
                </span>
              </div>
            {/each}
          </div>
        </div>
      {:else}
        <div class="py-16 text-center text-neutral-400 text-xs bg-white dark:bg-neutral-900/40 rounded-2xl border border-neutral-200 dark:border-neutral-800">
          Belum ada data audit. Silakan jalankan Security Audit terlebih dahulu.
        </div>
      {/if}
    </div>
  {/if}

  <!-- TAB 3: THREAT WATCHDOG (PRO) -->
  {#if activeTab === 'watchdog'}
    <div class="space-y-4">
      <div class="p-4 rounded-2xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900/40 space-y-4">
        <div class="flex items-center justify-between">
          <div>
            <div class="flex items-center gap-2">
              <h4 class="text-xs font-bold uppercase tracking-wider text-neutral-900 dark:text-white">Reverse Shell & Outbound Watchdog</h4>
              <span class="text-[10px] px-1.5 py-0.2 rounded bg-amber-500/20 text-amber-600 dark:text-amber-400 uppercase font-black">PRO</span>
            </div>
            <p class="text-xs text-neutral-500 mt-0.5">Mendeteksi proses siluman / socket outbound mencurigakan yang berpotensi menjadi backdoor atau reverse shell.</p>
          </div>
          <button
            onclick={startThreatWatchdog}
            disabled={isRunningWatchdog || !selectedHostId}
            class="px-3.5 py-1.5 bg-rose-600 hover:bg-rose-500 disabled:opacity-50 text-white text-xs font-semibold rounded-lg flex items-center gap-1.5 transition"
          >
            {#if isRunningWatchdog}
              <span class="animate-spin text-xs">⏳</span>
              <span>Scanning...</span>
            {:else}
              <span>🚨 Jalankan Watchdog</span>
            {/if}
          </button>
        </div>

        <ProGate title="Reverse Shell Threat Watchdog Pro" description="Pemantauan soket outbound berisiko tinggi dan deteksi reverse shell secara live membutuhkan CATerm Pro.">
          {#if watchdogResult}
            <div class="p-4 rounded-xl border {watchdogResult.threatLevel === 'CLEAN' ? 'border-emerald-500/30 bg-emerald-500/10 text-emerald-800 dark:text-emerald-300' : 'border-rose-500/30 bg-rose-500/10 text-rose-800 dark:text-rose-300'}">
              <div class="flex items-center justify-between">
                <span class="font-bold text-xs uppercase tracking-wider">Status Threat Watchdog: {watchdogResult.threatLevel}</span>
                <span class="text-[11px] opacity-80">{new Date(watchdogResult.timestamp).toLocaleTimeString()}</span>
              </div>
              <p class="text-xs mt-1">{watchdogResult.summary}</p>
              {#if watchdogResult.suspiciousSockets.length > 0}
                <div class="mt-3 space-y-1">
                  {#each watchdogResult.suspiciousSockets as s}
                    <div class="bg-black/20 p-2 rounded text-xs font-mono">
                      PID {s.pid} ({s.processName}): {s.exePath} &rarr; {s.remoteAddress || 'Local Socket'} [{s.reason}]
                    </div>
                  {/each}
                </div>
              {/if}
            </div>
          {:else}
            <div class="py-8 text-center text-neutral-400 text-xs">Tekan "Jalankan Watchdog" untuk memindai soket mencurigakan pada target.</div>
          {/if}
        </ProGate>
      </div>
    </div>
  {/if}

  <!-- TAB 4: TLS AUDITOR -->
  {#if activeTab === 'tls'}
    <div class="space-y-4">
      <div class="p-5 rounded-2xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900/60 space-y-4 max-w-xl">
        <div>
          <span class="text-xs font-bold text-neutral-900 dark:text-white uppercase tracking-wider">TLS Certificate Auditor</span>
          <p class="text-xs text-neutral-500 mt-0.5">Audit tanggal kedaluwarsa sertifikat SSL/TLS, cipher suite, dan status validitas rantai sertifikat.</p>
        </div>

        <div class="flex gap-2">
          <input
            type="text"
            bind:value={tlsHost}
            placeholder="domain.com"
            class="flex-1 px-3 py-2 text-xs bg-neutral-50 dark:bg-neutral-800 border border-neutral-300 dark:border-neutral-700 rounded-lg text-neutral-900 dark:text-white font-mono"
          />
          <button
            onclick={startTlsAudit}
            disabled={isRunningTls}
            class="px-4 py-2 bg-sky-600 text-white rounded-lg text-xs font-semibold hover:bg-sky-500 disabled:opacity-50 transition"
          >
            {isRunningTls ? 'Checking...' : 'Audit TLS'}
          </button>
        </div>

        {#if tlsResult}
          <div class="p-4 rounded-xl border border-neutral-200 dark:border-neutral-700 bg-neutral-50 dark:bg-neutral-800/40 space-y-2 text-xs font-mono">
            <div class="flex justify-between">
              <span class="text-neutral-500">Status Validitas:</span>
              <strong class="{tlsResult.status === 'HEALTHY' ? 'text-emerald-500' : 'text-rose-500'}">{tlsResult.status}</strong>
            </div>
            <div class="flex justify-between">
              <span class="text-neutral-500">Sisa Masa Aktif:</span>
              <strong>{tlsResult.daysRemaining} hari</strong>
            </div>
            <div class="pt-2 border-t border-neutral-200 dark:border-neutral-700 text-[11px] text-neutral-400">
              Cipher Suite: {tlsResult.cipherSuite}
            </div>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>
