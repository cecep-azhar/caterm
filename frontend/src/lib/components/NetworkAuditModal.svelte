<script lang="ts">
  import { onMount } from 'svelte';
  import { runNetworkSecurityAudit, type AuditReport } from '$lib/api/audit';
  import { listHosts, type HostRecord } from '$lib/api/hosts';
  import { saveInvestigation } from '$lib/api/investigations';
  import { showToast } from '$lib/stores/uiNotifications.svelte';
  import { errorText } from '$lib/errors';
  import { t } from '$lib/i18n/index.svelte';

  let { hostId, onClose }: { hostId?: string; onClose: () => void } = $props();

  let hosts = $state<HostRecord[]>([]);
  let selectedHostId = $state<string>(hostId || '');
  let isRunning = $state(false);
  let report = $state<AuditReport | null>(null);
  let activeTab = $state<'overview' | 'egress' | 'inbound' | 'probe' | 'hardening'>('overview');
  let isSavingIncident = $state(false);

  onMount(async () => {
    try {
      hosts = await listHosts();
      if (!selectedHostId && hosts.length > 0) {
        selectedHostId = hosts[0].id;
      }
    } catch (e) {
      // ignore
    }
  });

  async function startAudit() {
    if (!selectedHostId) {
      showToast('Pilih host target terlebih dahulu', 'error');
      return;
    }
    isRunning = true;
    report = null;
    try {
      const res = await runNetworkSecurityAudit(selectedHostId);
      report = res;
      showToast(`Audit selesai! Skor Keamanan: ${res.summaryScore}%`, res.summaryScore > 70 ? 'success' : 'error');
    } catch (e) {
      showToast(`Audit gagal: ${errorText(e)}`, 'error');
    } finally {
      isRunning = false;
    }
  }

  async function exportToInvestigation() {
    if (!report) return;
    isSavingIncident = true;
    try {
      const title = `Security Audit Report - Host ${getHostLabel(report.hostId)} (Skor: ${report.summaryScore}%)`;
      const notes = `Otomatis diekspor dari DevOps Network & Security Audit Hub.
Skor Keamanan: ${report.summaryScore}%
Temuan Inbound Wildcard: ${report.inboundPorts.filter(p => p.riskLevel === 'CRITICAL' || p.riskLevel === 'HIGH').length} port
Temuan Egress Leak: ${report.egressResults.filter(e => e.status === 'OPEN_LEAK').length} port
SSH Hardening Fail/Warn: ${report.hardeningChecklist.filter(h => h.status !== 'PASS').length} check`;

      await saveInvestigation({
        title,
        host_id: report.hostId,
        status: report.summaryScore < 70 ? 'OPEN' : 'CLOSED',
        notes,
        evidence: JSON.stringify(report, null, 2)
      });
      showToast('Laporan berhasil disimpan ke Investigasi Insiden!', 'success');
    } catch (e) {
      showToast(`Gagal menyimpan investigasi: ${errorText(e)}`, 'error');
    } finally {
      isSavingIncident = false;
    }
  }

  function getHostLabel(id: string): string {
    const found = hosts.find(h => h.id === id);
    return found ? found.label || found.address : id;
  }
</script>

<div class="fixed inset-0 bg-black/60 backdrop-blur-sm z-50 flex items-center justify-center p-4">
  <div class="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-xl shadow-2xl max-w-4xl w-full max-h-[90vh] flex flex-col overflow-hidden">
    <!-- Header -->
    <div class="px-6 py-4 border-b border-neutral-200 dark:border-neutral-800 flex items-center justify-between bg-neutral-50 dark:bg-neutral-900/50">
      <div class="flex items-center gap-3">
        <div class="w-10 h-10 rounded-lg bg-sky-500/10 text-sky-600 dark:text-sky-400 flex items-center justify-center border border-sky-500/20">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"/>
          </svg>
        </div>
        <div>
          <h2 class="text-lg font-bold text-neutral-900 dark:text-white">DevOps Network & Security Audit Hub</h2>
          <p class="text-xs text-neutral-500">Inbound exposure, egress port leaks, latency probe & host hardening audit</p>
        </div>
      </div>
      <button 
        onclick={onClose}
        class="text-neutral-400 hover:text-neutral-600 dark:hover:text-neutral-200 p-1.5 rounded-lg hover:bg-neutral-100 dark:hover:bg-neutral-800 transition"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/></svg>
      </button>
    </div>

    <!-- Controls Bar -->
    <div class="px-6 py-3 border-b border-neutral-200 dark:border-neutral-800 flex flex-wrap items-center justify-between gap-3 bg-neutral-50/50 dark:bg-neutral-950/40">
      <div class="flex items-center gap-3">
        <label for="audit-host-select" class="text-xs font-medium text-neutral-500 uppercase">Target Host:</label>
        <select 
          id="audit-host-select"
          bind:value={selectedHostId}
          disabled={isRunning}
          class="px-3 py-1.5 text-xs bg-white dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-700 rounded-lg text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500"
        >
          <option value="">-- Pilih Host --</option>
          {#each hosts as h}
            <option value={h.id}>{h.label} ({h.address})</option>
          {/each}
        </select>
        <button
          onclick={startAudit}
          disabled={isRunning || !selectedHostId}
          class="px-4 py-1.5 bg-sky-600 hover:bg-sky-500 disabled:opacity-50 text-white text-xs font-semibold rounded-lg shadow shadow-sky-600/20 flex items-center gap-2 transition"
        >
          {#if isRunning}
            <svg class="w-3.5 h-3.5 animate-spin" fill="none" viewBox="0 0 24 24"><circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle><path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v8H4z"></path></svg>
            <span>Sedang Mengaudit...</span>
          {:else}
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"/></svg>
            <span>Mulai Audit 1-Klik</span>
          {/if}
        </button>
      </div>

      {#if report}
        <button
          onclick={exportToInvestigation}
          disabled={isSavingIncident}
          class="px-3 py-1.5 bg-neutral-200 dark:bg-neutral-800 hover:bg-neutral-300 dark:hover:bg-neutral-700 text-xs font-medium rounded-lg text-neutral-800 dark:text-neutral-200 flex items-center gap-1.5 transition"
        >
          <svg class="w-3.5 h-3.5 text-rose-500" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"/></svg>
          <span>{isSavingIncident ? 'Menyimpan...' : 'Ekspor ke Investigasi'}</span>
        </button>
      {/if}
    </div>

    <!-- Content Area -->
    <div class="flex-1 overflow-y-auto p-6 space-y-6">
      {#if !report && !isRunning}
        <div class="py-16 text-center space-y-3">
          <div class="w-16 h-16 rounded-2xl bg-neutral-100 dark:bg-neutral-800 flex items-center justify-center mx-auto text-neutral-400">
            <svg class="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"/></svg>
          </div>
          <h3 class="text-base font-semibold text-neutral-800 dark:text-neutral-200">Belum Ada Audit yang Dijalankan</h3>
          <p class="text-xs text-neutral-500 max-w-md mx-auto">
            Pilih host target di atas dan tekan <strong>"Mulai Audit 1-Klik"</strong> untuk menguji port egress leak, exposure port listening 0.0.0.0, kualitas latency MTR, serta hardening SSH.
          </p>
        </div>
      {:else if isRunning}
        <div class="py-16 text-center space-y-4">
          <div class="w-12 h-12 rounded-full border-4 border-sky-500/20 border-t-sky-500 animate-spin mx-auto"></div>
          <div class="space-y-1">
            <h4 class="text-sm font-semibold text-neutral-800 dark:text-neutral-200">Menjalankan Automated Security Scanner...</h4>
            <p class="text-xs text-neutral-500">Mengecek konektivitas keluar TCP, port ss/netstat, routing probe, dan konfigurasi sshd_config.</p>
          </div>
        </div>
      {:else if report}
        <!-- Score & Highlights Banner -->
        <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
          <div class="p-4 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-900 flex flex-col justify-between">
            <span class="text-xs font-semibold text-neutral-500 uppercase">Skor Keamanan</span>
            <div class="flex items-baseline gap-2 mt-2">
              <span class="text-3xl font-black {report.summaryScore >= 80 ? 'text-emerald-500' : report.summaryScore >= 60 ? 'text-amber-500' : 'text-rose-500'}">
                {report.summaryScore}%
              </span>
              <span class="text-xs text-neutral-400 font-medium">
                {report.summaryScore >= 80 ? 'BAIK' : report.summaryScore >= 60 ? 'PERINGATAN' : 'RENTAN'}
              </span>
            </div>
            <div class="w-full bg-neutral-200 dark:bg-neutral-800 h-1.5 rounded-full mt-3 overflow-hidden">
              <div 
                class="h-full rounded-full transition-all {report.summaryScore >= 80 ? 'bg-emerald-500' : report.summaryScore >= 60 ? 'bg-amber-500' : 'bg-rose-500'}"
                style="width: {report.summaryScore}%"
              ></div>
            </div>
          </div>

          {#if report}
            {@const leaks = report.egressResults.filter(e => e.status === 'OPEN_LEAK')}
            {@const exposed = report.inboundPorts.filter(p => p.isWildcard && (p.riskLevel === 'CRITICAL' || p.riskLevel === 'HIGH'))}
            <div class="p-4 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-900 flex flex-col justify-between">
              <span class="text-xs font-semibold text-neutral-500 uppercase">Egress Port Leaks</span>
              <div class="mt-2">
                <span class="text-2xl font-bold {leaks.length > 0 ? 'text-rose-500' : 'text-emerald-500'}">
                  {leaks.length}
                </span>
                <span class="text-xs text-neutral-500 ml-1">terbuka keluar</span>
              </div>
              <p class="text-[11px] text-neutral-400 mt-2">Port krusial (SMTP, DB, Redis) dicek</p>
            </div>

            <div class="p-4 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-900 flex flex-col justify-between">
              <span class="text-xs font-semibold text-neutral-500 uppercase">Inbound Wildcard (0.0.0.0)</span>
              <div class="mt-2">
                <span class="text-2xl font-bold {exposed.length > 0 ? 'text-amber-500' : 'text-emerald-500'}">
                  {exposed.length}
                </span>
                <span class="text-xs text-neutral-500 ml-1">berisiko terekspos</span>
              </div>
              <p class="text-[11px] text-neutral-400 mt-2">Listening sockets tanpa isolasi</p>
            </div>
          {/if}

          <div class="p-4 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-900 flex flex-col justify-between">
            <span class="text-xs font-semibold text-neutral-500 uppercase">Latency & Jitter</span>
            <div class="mt-2">
              <span class="text-2xl font-bold text-sky-500">
                {report.latencyProbe.avgLatencyMs.toFixed(1)} ms
              </span>
              <span class="text-xs text-neutral-500 ml-1">({report.latencyProbe.packetLossPct}% loss)</span>
            </div>
            <p class="text-[11px] text-neutral-400 mt-2">Probe ke 1.1.1.1 ({report.latencyProbe.status})</p>
          </div>
        </div>

        <!-- Navigation Tabs -->
        <div class="flex border-b border-neutral-200 dark:border-neutral-800 gap-6 text-xs font-semibold">
          <button 
            onclick={() => activeTab = 'overview'}
            class="pb-2.5 transition border-b-2 {activeTab === 'overview' ? 'border-sky-500 text-sky-600 dark:text-sky-400' : 'border-transparent text-neutral-500 hover:text-neutral-800 dark:hover:text-neutral-200'}"
          >
            Ringkasan & Temuan
          </button>
          <button 
            onclick={() => activeTab = 'egress'}
            class="pb-2.5 transition border-b-2 {activeTab === 'egress' ? 'border-sky-500 text-sky-600 dark:text-sky-400' : 'border-transparent text-neutral-500 hover:text-neutral-800 dark:hover:text-neutral-200'}"
          >
            Egress Port Hunter ({report.egressResults.length})
          </button>
          <button 
            onclick={() => activeTab = 'inbound'}
            class="pb-2.5 transition border-b-2 {activeTab === 'inbound' ? 'border-sky-500 text-sky-600 dark:text-sky-400' : 'border-transparent text-neutral-500 hover:text-neutral-800 dark:hover:text-neutral-200'}"
          >
            Inbound Exposure ({report.inboundPorts.length})
          </button>
          <button 
            onclick={() => activeTab = 'probe'}
            class="pb-2.5 transition border-b-2 {activeTab === 'probe' ? 'border-sky-500 text-sky-600 dark:text-sky-400' : 'border-transparent text-neutral-500 hover:text-neutral-800 dark:hover:text-neutral-200'}"
          >
            Network Benchmark & Latency
          </button>
          <button 
            onclick={() => activeTab = 'hardening'}
            class="pb-2.5 transition border-b-2 {activeTab === 'hardening' ? 'border-sky-500 text-sky-600 dark:text-sky-400' : 'border-transparent text-neutral-500 hover:text-neutral-800 dark:hover:text-neutral-200'}"
          >
            SSH Hardening Checklist ({report.hardeningChecklist.length})
          </button>
        </div>

        <!-- Tab Content -->
        {#if activeTab === 'overview'}
          <div class="space-y-4">
            <h4 class="text-xs font-bold uppercase text-neutral-500 tracking-wider">Checklist Rekomendasi Penting</h4>
            <div class="space-y-2">
              {#each report.hardeningChecklist as item}
                <div class="p-3.5 rounded-lg border border-neutral-200 dark:border-neutral-800 bg-neutral-50/50 dark:bg-neutral-900/40 flex items-start justify-between gap-4">
                  <div>
                    <div class="flex items-center gap-2">
                      <span class="text-xs font-bold text-neutral-800 dark:text-neutral-200">{item.name}</span>
                      <span class="text-[10px] px-1.5 py-0.5 rounded font-bold {item.status === 'PASS' ? 'bg-emerald-500/20 text-emerald-600 dark:text-emerald-400' : item.status === 'WARN' ? 'bg-amber-500/20 text-amber-600 dark:text-amber-400' : 'bg-rose-500/20 text-rose-600 dark:text-rose-400'}">
                        {item.status}
                      </span>
                    </div>
                    <p class="text-xs text-neutral-500 mt-1">{item.details}</p>
                  </div>
                  <div class="text-right text-xs shrink-0">
                    <span class="text-neutral-400">Aktual:</span> <code class="font-mono text-neutral-700 dark:text-neutral-300">{item.currentValue}</code>
                    <div class="text-[11px] text-neutral-500">Rekomendasi: {item.recommendedValue}</div>
                  </div>
                </div>
              {/each}
            </div>
          </div>
        {:else if activeTab === 'egress'}
          <div class="space-y-3">
            <p class="text-xs text-neutral-500">Mendeteksi apakah firewall outbound server membiarkan port krusial keluar bebas.</p>
            <div class="border border-neutral-200 dark:border-neutral-800 rounded-lg overflow-hidden">
              <table class="w-full text-xs text-left">
                <thead class="bg-neutral-100 dark:bg-neutral-800/60 text-neutral-500 uppercase text-[10px]">
                  <tr>
                    <th class="p-3">Port & Layanan</th>
                    <th class="p-3">Target Probe</th>
                    <th class="p-3">Status</th>
                    <th class="p-3">Tingkat Risiko</th>
                    <th class="p-3">Catatan</th>
                  </tr>
                </thead>
                <tbody class="divide-y divide-neutral-200 dark:divide-neutral-800 font-mono">
                  {#each report.egressResults as egress}
                    <tr>
                      <td class="p-3 font-semibold text-neutral-900 dark:text-white">{egress.port} ({egress.serviceLabel})</td>
                      <td class="p-3 text-neutral-600 dark:text-neutral-400">{egress.target}</td>
                      <td class="p-3">
                        <span class="px-2 py-0.5 rounded text-[10px] font-bold {egress.status === 'OPEN_LEAK' ? 'bg-rose-500/20 text-rose-600 dark:text-rose-400' : 'bg-emerald-500/20 text-emerald-600 dark:text-emerald-400'}">
                          {egress.status}
                        </span>
                      </td>
                      <td class="p-3 text-[11px] {egress.riskLevel === 'CRITICAL' ? 'text-rose-500 font-bold' : egress.riskLevel === 'HIGH' ? 'text-amber-500' : 'text-neutral-400'}">
                        {egress.riskLevel}
                      </td>
                      <td class="p-3 font-sans text-neutral-500 text-[11px]">{egress.notes}</td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          </div>
        {:else if activeTab === 'inbound'}
          <div class="space-y-3">
            <p class="text-xs text-neutral-500">Daftar service listening socket pada server. Waspadai port database/redis yang bind ke 0.0.0.0 publik.</p>
            <div class="border border-neutral-200 dark:border-neutral-800 rounded-lg overflow-hidden">
              <table class="w-full text-xs text-left">
                <thead class="bg-neutral-100 dark:bg-neutral-800/60 text-neutral-500 uppercase text-[10px]">
                  <tr>
                    <th class="p-3">Proto</th>
                    <th class="p-3">Alamat Listening</th>
                    <th class="p-3">Port</th>
                    <th class="p-3">Proses</th>
                    <th class="p-3">Risiko</th>
                    <th class="p-3">Rekomendasi</th>
                  </tr>
                </thead>
                <tbody class="divide-y divide-neutral-200 dark:divide-neutral-800 font-mono">
                  {#each report.inboundPorts as p}
                    <tr>
                      <td class="p-3 text-neutral-500 uppercase">{p.proto}</td>
                      <td class="p-3 font-semibold {p.isWildcard ? 'text-amber-600 dark:text-amber-400' : 'text-neutral-700 dark:text-neutral-300'}">{p.localAddress}</td>
                      <td class="p-3 font-bold text-neutral-900 dark:text-white">{p.port || '-'}</td>
                      <td class="p-3 text-neutral-600 dark:text-neutral-400">{p.process}</td>
                      <td class="p-3">
                        <span class="px-2 py-0.5 rounded text-[10px] font-bold {p.riskLevel === 'CRITICAL' ? 'bg-rose-500/20 text-rose-600 dark:text-rose-400' : p.riskLevel === 'HIGH' ? 'bg-amber-500/20 text-amber-600 dark:text-amber-400' : 'bg-neutral-100 dark:bg-neutral-800 text-neutral-500'}">
                          {p.riskLevel}
                        </span>
                      </td>
                      <td class="p-3 font-sans text-neutral-500 text-[11px]">{p.recommendation}</td>
                    </tr>
                  {/each}
                  {#if report.inboundPorts.length === 0}
                    <tr><td colspan="6" class="p-4 text-center text-neutral-500">Tidak ada listening socket terdeteksi.</td></tr>
                  {/if}
                </tbody>
              </table>
            </div>
          </div>
        {:else if activeTab === 'probe'}
          <div class="space-y-4">
            <div class="grid grid-cols-3 gap-4">
              <div class="p-4 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-900">
                <span class="text-[11px] text-neutral-400 uppercase font-medium">Packet Loss</span>
                <p class="text-xl font-bold {report.latencyProbe.packetLossPct === 0 ? 'text-emerald-500' : 'text-rose-500'} mt-1">
                  {report.latencyProbe.packetLossPct}%
                </p>
              </div>
              <div class="p-4 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-900">
                <span class="text-[11px] text-neutral-400 uppercase font-medium">Min / Avg / Max Latency</span>
                <p class="text-xl font-bold text-sky-500 mt-1">
                  {report.latencyProbe.minLatencyMs.toFixed(1)} / {report.latencyProbe.avgLatencyMs.toFixed(1)} / {report.latencyProbe.maxLatencyMs.toFixed(1)} ms
                </p>
              </div>
              <div class="p-4 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-900">
                <span class="text-[11px] text-neutral-400 uppercase font-medium">Kualitas Rute Jaringan</span>
                <p class="text-xl font-bold text-emerald-500 mt-1">
                  {report.latencyProbe.status}
                </p>
              </div>
            </div>

            <div class="border border-neutral-200 dark:border-neutral-800 rounded-lg p-4 bg-neutral-900 text-neutral-300 font-mono text-xs space-y-1">
              <span class="text-neutral-500 block mb-2">// Routing Hops (traceroute / tracepath):</span>
              {#each report.latencyProbe.hops as hop}
                <div>{hop}</div>
              {/each}
              {#if report.latencyProbe.hops.length === 0}
                <div class="text-neutral-600">Tidak ada output hop routing.</div>
              {/if}
            </div>
          </div>
        {:else if activeTab === 'hardening'}
          <div class="space-y-3">
            <p class="text-xs text-neutral-500">Evaluasi kepatuhan konfigurasi SSH dan Firewall server terhadap standar keamanan Linux.</p>
            <div class="space-y-2">
              {#each report.hardeningChecklist as h}
                <div class="p-3.5 rounded-lg border border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-900/50 flex items-center justify-between">
                  <div class="space-y-1">
                    <div class="flex items-center gap-2">
                      <span class="text-xs font-bold text-neutral-900 dark:text-white">{h.name}</span>
                      <span class="text-[10px] px-2 py-0.5 rounded font-bold {h.status === 'PASS' ? 'bg-emerald-500/20 text-emerald-600 dark:text-emerald-400' : h.status === 'WARN' ? 'bg-amber-500/20 text-amber-600 dark:text-amber-400' : 'bg-rose-500/20 text-rose-600 dark:text-rose-400'}">
                        {h.status}
                      </span>
                    </div>
                    <p class="text-xs text-neutral-500">{h.details}</p>
                  </div>
                  <div class="text-right text-xs">
                    <span class="text-neutral-400">Aktual:</span> <code class="font-mono">{h.currentValue}</code>
                  </div>
                </div>
              {/each}
            </div>
          </div>
        {/if}
      {/if}
    </div>

    <!-- Footer -->
    <div class="px-6 py-3 border-t border-neutral-200 dark:border-neutral-800 flex justify-end bg-neutral-50/50 dark:bg-neutral-900/50">
      <button 
        onclick={onClose}
        class="px-4 py-2 bg-neutral-200 dark:bg-neutral-800 hover:bg-neutral-300 dark:hover:bg-neutral-700 text-xs font-semibold rounded-lg text-neutral-800 dark:text-neutral-200 transition"
      >
        Tutup
      </button>
    </div>
  </div>
</div>
