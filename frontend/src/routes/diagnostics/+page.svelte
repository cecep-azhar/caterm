<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import {
    runNetworkSecurityAudit,
    runServerBenchmark,
    runNetworkSpeedTest,
    runQosAudit,
    runMeshLatencyMatrix,
    runPmtudProbe,
    runSubnetSweep,
    runThreatWatchdog,
    runTlsAudit,
    type AuditReport,
    type ServerBenchmarkResult,
    type SpeedTestResult,
    type QosAuditResult,
    type MeshLatencyMatrix,
    type PmtudResult,
    type SubnetSweepResult,
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
  let activeTab = $state<'security' | 'speed_qos' | 'benchmark' | 'mesh_tools'>('security');

  // Loading States
  let isRunningSecurity = $state(false);
  let isRunningSpeed = $state(false);
  let isRunningQos = $state(false);
  let isRunningBench = $state(false);
  let isRunningMesh = $state(false);
  let isRunningSubnet = $state(false);
  let isRunningPmtud = $state(false);
  let isRunningWatchdog = $state(false);
  let isRunningTls = $state(false);
  let isSavingIncident = $state(false);

  // Results State
  let securityReport = $state<AuditReport | null>(null);
  let speedResult = $state<SpeedTestResult | null>(null);
  let qosResult = $state<QosAuditResult | null>(null);
  let benchResult = $state<ServerBenchmarkResult | null>(null);
  let meshResult = $state<MeshLatencyMatrix | null>(null);
  let subnetResult = $state<SubnetSweepResult | null>(null);
  let pmtudResult = $state<PmtudResult | null>(null);
  let watchdogResult = $state<ThreatWatchdogResult | null>(null);
  let tlsResult = $state<TlsAuditResult | null>(null);

  // Subnet & Tools inputs
  let subnetCidr = $state('192.168.1.0/24');
  let pmtudTarget = $state('1.1.1.1');
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

  // Action Handlers
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

  async function startSpeedTest() {
    if (!selectedHostId) return;
    isRunningSpeed = true;
    try {
      speedResult = await runNetworkSpeedTest(selectedHostId);
      showToast(`Speed Test selesai: Down ${speedResult.downloadMbps} Mbps, Up ${speedResult.uploadMbps} Mbps`, 'success');
    } catch (e) {
      showToast(`Speed Test gagal: ${errorText(e)}`, 'error');
    } finally {
      isRunningSpeed = false;
    }
  }

  async function startQosAudit() {
    if (!selectedHostId) return;
    isRunningQos = true;
    try {
      qosResult = await runQosAudit(selectedHostId);
      showToast(`QoS Audit selesai: Grade ${qosResult.bufferbloatGrade}`, 'success');
    } catch (e) {
      showToast(`QoS Audit gagal: ${errorText(e)}`, 'error');
    } finally {
      isRunningQos = false;
    }
  }

  async function startBenchmark() {
    if (!selectedHostId) return;
    isRunningBench = true;
    try {
      benchResult = await runServerBenchmark(selectedHostId);
      showToast(`Server benchmark selesai! RAM: ${benchResult.ramBandwidthMbps} MB/s`, 'success');
    } catch (e) {
      showToast(`Benchmark gagal: ${errorText(e)}`, 'error');
    } finally {
      isRunningBench = false;
    }
  }

  async function startThreatWatchdog() {
    if (!selectedHostId) return;
    isRunningWatchdog = true;
    try {
      watchdogResult = await runThreatWatchdog(selectedHostId);
      showToast(
        `Watchdog scan: ${watchdogResult.threatLevel}`,
        watchdogResult.threatLevel === 'CLEAN' ? 'success' : 'error'
      );
    } catch (e) {
      showToast(`Watchdog scan gagal: ${errorText(e)}`, 'error');
    } finally {
      isRunningWatchdog = false;
    }
  }

  async function startMeshMatrix() {
    if (!selectedHostId) return;
    isRunningMesh = true;
    try {
      const targets: [string, string][] = hosts
        .filter((h) => h.id !== selectedHostId)
        .map((h) => [h.address, h.label || h.address]);

      if (targets.length === 0) {
        targets.push(['1.1.1.1', 'Cloudflare DNS'], ['8.8.8.8', 'Google DNS']);
      }
      meshResult = await runMeshLatencyMatrix(selectedHostId, targets);
      showToast('Mesh Latency Matrix selesai diperbarui', 'success');
    } catch (e) {
      showToast(`Mesh Latency Matrix gagal: ${errorText(e)}`, 'error');
    } finally {
      isRunningMesh = false;
    }
  }

  async function startSubnetSweep() {
    if (!selectedHostId) return;
    isRunningSubnet = true;
    try {
      subnetResult = await runSubnetSweep(selectedHostId, subnetCidr);
      showToast(`Subnet Sweep selesai: Ditemukan ${subnetResult.activeHosts} host aktif`, 'success');
    } catch (e) {
      showToast(`Subnet sweep gagal: ${errorText(e)}`, 'error');
    } finally {
      isRunningSubnet = false;
    }
  }

  async function startPmtudProbe() {
    if (!selectedHostId) return;
    isRunningPmtud = true;
    try {
      pmtudResult = await runPmtudProbe(selectedHostId, pmtudTarget);
      showToast(`PMTUD optimal MTU: ${pmtudResult.optimalMtu}`, 'success');
    } catch (e) {
      showToast(`PMTUD probe gagal: ${errorText(e)}`, 'error');
    } finally {
      isRunningPmtud = false;
    }
  }

  async function startTlsAudit() {
    isRunningTls = true;
    try {
      tlsResult = await runTlsAudit(tlsHost, tlsPort);
      showToast(`TLS Audit: ${tlsResult.status} (${tlsResult.daysRemaining} hari sisa)`, 'success');
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
      const title = `DevOps Security & Lab Report - Host ${getHostLabel(securityReport.hostId)} (Skor: ${securityReport.summaryScore}%)`;
      const notes = `Otomatis diekspor dari DevOps Diagnostics & Security Lab Suite.
Skor Keamanan: ${securityReport.summaryScore}%
Temuan Inbound Wildcard: ${securityReport.inboundPorts.filter((p) => p.riskLevel === 'CRITICAL' || p.riskLevel === 'HIGH').length} port
Temuan Egress Leak: ${securityReport.egressResults.filter((e) => e.status === 'OPEN_LEAK').length} port
SSH Hardening Fail/Warn: ${securityReport.hardeningChecklist.filter((h) => h.status !== 'PASS').length} check`;

      await saveInvestigation({
        title,
        host_id: securityReport.hostId === 'local' ? '' : securityReport.hostId,
        status: securityReport.summaryScore < 70 ? 'OPEN' : 'CLOSED',
        notes,
        evidence: JSON.stringify(
          {
            security: securityReport,
            benchmark: benchResult,
            speed: speedResult,
            qos: qosResult,
            watchdog: watchdogResult
          },
          null,
          2
        )
      });
      showToast('Laporan berhasil disimpan ke Investigasi Insiden!', 'success');
    } catch (e) {
      showToast(`Gagal menyimpan investigasi: ${errorText(e)}`, 'error');
    } finally {
      isSavingIncident = false;
    }
  }

  function getHostLabel(id: string): string {
    if (id === 'local') return 'Mesin Lokal';
    const found = hosts.find((h) => h.id === id);
    return found ? found.label || found.address : id;
  }
</script>

<svelte:head>
  <title>DevOps Lab & Diagnostics · CATerm</title>
</svelte:head>

<div class="max-w-6xl mx-auto space-y-6">
  <PageHeader
    icon={['M19.428 15.428a2 2 0 00-1.022-.547l-2.387-.477a6 6 0 00-3.86.438l-.318.131a6 6 0 01-3.86.438l-2.387-.477a2 2 0 00-1.022.547l-1.89 1.89A2 2 0 002.5 19v1.5a1.5 1.5 0 001.5 1.5h16a1.5 1.5 0 001.5-1.5V19a2 2 0 00-.598-1.414l-1.474-1.474zM12 2v6m0 0a3 3 0 100 6 3 3 0 000-6z']}
    accent="amber"
    title="DevOps Diagnostics & Lab Suite"
    badge="Lab Suite"
    subtitle="Security Exposure, Threat Watchdog, Speed & QoS Bufferbloat, Hardware Benchmark, dan Mesh Armada Matrix."
  >
    {#snippet actions()}
      <div class="flex items-center gap-2 flex-wrap">
        <label for="diag-host-select" class="text-xs font-semibold text-neutral-500 uppercase tracking-wider">Host:</label>
        <select
          id="diag-host-select"
          bind:value={selectedHostId}
          class="px-3 py-1.5 text-xs bg-white dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-700 rounded-xl text-neutral-900 dark:text-white focus:outline-none focus:border-amber-500 shadow-sm font-medium"
        >
          <option value="">-- Pilih Target Host --</option>
          <option value="local">🖥️ Mesin Lokal (Localhost CLI)</option>
          {#each hosts as h}
            <option value={h.id}>{h.label || h.address} ({h.address})</option>
          {/each}
        </select>

        {#if securityReport}
          <button
            onclick={exportToInvestigation}
            disabled={isSavingIncident}
            class="px-3 py-1.5 bg-neutral-200 dark:bg-neutral-800 hover:bg-neutral-300 dark:hover:bg-neutral-700 text-xs font-medium rounded-xl text-neutral-800 dark:text-neutral-200 flex items-center gap-1.5 transition"
          >
            <svg class="w-3.5 h-3.5 text-rose-500" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"/></svg>
            <span>{isSavingIncident ? 'Menyimpan...' : 'Ekspor Laporan'}</span>
          </button>
        {/if}
      </div>
    {/snippet}
  </PageHeader>

  <!-- Main Tabs Switcher -->
  <div class="flex items-center gap-1.5 bg-neutral-100 dark:bg-neutral-900/60 p-1.5 rounded-2xl border border-neutral-200 dark:border-neutral-800 text-xs font-semibold overflow-x-auto">
    <button
      onclick={() => (activeTab = 'security')}
      class="px-4 py-2 rounded-xl transition flex items-center gap-2 whitespace-nowrap {activeTab === 'security' ? 'bg-white dark:bg-neutral-800 text-sky-600 dark:text-sky-400 shadow-sm border border-neutral-200/50 dark:border-neutral-700' : 'text-neutral-600 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white'}"
    >
      <span>🛡️</span>
      <span>Security & Hole Hunter</span>
    </button>
    <button
      onclick={() => (activeTab = 'speed_qos')}
      class="px-4 py-2 rounded-xl transition flex items-center gap-2 whitespace-nowrap {activeTab === 'speed_qos' ? 'bg-white dark:bg-neutral-800 text-sky-600 dark:text-sky-400 shadow-sm border border-neutral-200/50 dark:border-neutral-700' : 'text-neutral-600 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white'}"
    >
      <span>⚡</span>
      <span>Speed Test & QoS</span>
    </button>
    <button
      onclick={() => (activeTab = 'benchmark')}
      class="px-4 py-2 rounded-xl transition flex items-center gap-2 whitespace-nowrap {activeTab === 'benchmark' ? 'bg-white dark:bg-neutral-800 text-amber-600 dark:text-amber-400 shadow-sm border border-neutral-200/50 dark:border-neutral-700' : 'text-neutral-600 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white'}"
    >
      <span>🔥</span>
      <span>Server Benchmark</span>
      <span class="text-[10px] px-1.5 py-0.2 rounded bg-amber-500/20 text-amber-600 dark:text-amber-400 uppercase font-black">PRO</span>
    </button>
    <button
      onclick={() => (activeTab = 'mesh_tools')}
      class="px-4 py-2 rounded-xl transition flex items-center gap-2 whitespace-nowrap {activeTab === 'mesh_tools' ? 'bg-white dark:bg-neutral-800 text-indigo-600 dark:text-indigo-400 shadow-sm border border-neutral-200/50 dark:border-neutral-700' : 'text-neutral-600 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white'}"
    >
      <span>🌐</span>
      <span>Mesh Matrix & Tools</span>
      <span class="text-[10px] px-1.5 py-0.2 rounded bg-amber-500/20 text-amber-600 dark:text-amber-400 uppercase font-black">PRO</span>
    </button>
  </div>

  <!-- TAB CONTENT -->
  <!-- TAB 1: SECURITY & HOLE HUNTER (Threat watchdog wrapped in ProGate) -->
  {#if activeTab === 'security'}
    <div class="space-y-6">
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 p-4 rounded-2xl bg-white dark:bg-neutral-900/60 border border-neutral-200 dark:border-neutral-800">
        <div>
          <h3 class="text-sm font-bold text-neutral-900 dark:text-white">Security Exposure & Hole Hunter</h3>
          <p class="text-xs text-neutral-500">Pemindaian port listening wildcard 0.0.0.0, kebocoran egress outbound, dan baseline hardening SSH.</p>
        </div>
        <div class="flex items-center gap-2">
          <button
            onclick={startSecurityAudit}
            disabled={isRunningSecurity || !selectedHostId}
            class="px-4 py-2 bg-sky-600 hover:bg-sky-500 disabled:opacity-50 text-white text-xs font-semibold rounded-xl shadow-md shadow-sky-600/20 flex items-center gap-2 transition"
          >
            {#if isRunningSecurity}
              <span class="animate-spin text-xs">⏳</span>
              <span>Mengaudit Keamanan...</span>
            {:else}
              <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"/></svg>
              <span>Jalankan Security Audit</span>
            {/if}
          </button>
        </div>
      </div>

      <!-- Threat Watchdog PRO Guard -->
      <div class="p-4 rounded-2xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900/40 space-y-4">
        <div class="flex items-center justify-between">
          <div>
            <div class="flex items-center gap-2">
              <h4 class="text-xs font-bold uppercase tracking-wider text-neutral-900 dark:text-white">Reverse Shell & Outbound Watchdog</h4>
              <span class="text-[10px] px-1.5 py-0.2 rounded bg-amber-500/20 text-amber-600 dark:text-amber-400 uppercase font-black">PRO</span>
            </div>
            <p class="text-xs text-neutral-500">Mendeteksi proses siluman / socket outbound mencurigakan yang berpotensi menjadi reverse shell.</p>
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

      <!-- General Security Results -->
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
            <span class="text-xs font-semibold text-neutral-500 uppercase">Latency Ping</span>
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
          Tekan "Jalankan Security Audit" untuk memindai exposure port dan hardening SSH host.
        </div>
      {/if}
    </div>
  {/if}

  <!-- TAB 2: SPEED TEST & QOS -->
  {#if activeTab === 'speed_qos'}
    <div class="space-y-6">
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 p-4 rounded-2xl bg-white dark:bg-neutral-900/60 border border-neutral-200 dark:border-neutral-800">
        <div>
          <h3 class="text-sm font-bold text-neutral-900 dark:text-white">Throughput & QoS Bufferbloat Auditor</h3>
          <p class="text-xs text-neutral-500">Mengukur bandwidth download/upload real-time dan degradasi latensi di bawah beban (Bufferbloat Grade A+ hingga F).</p>
        </div>
        <div class="flex items-center gap-2">
          <button
            onclick={startSpeedTest}
            disabled={isRunningSpeed || !selectedHostId}
            class="px-4 py-2 bg-sky-600 hover:bg-sky-500 disabled:opacity-50 text-white text-xs font-semibold rounded-xl shadow-md shadow-sky-600/20 flex items-center gap-2 transition"
          >
            {#if isRunningSpeed}
              <span class="animate-spin text-xs">⏳</span>
              <span>Testing Speed...</span>
            {:else}
              <span>🚀 Test Speedometer</span>
            {/if}
          </button>
          <button
            onclick={startQosAudit}
            disabled={isRunningQos || !selectedHostId}
            class="px-4 py-2 bg-emerald-600 hover:bg-emerald-500 disabled:opacity-50 text-white text-xs font-semibold rounded-xl shadow-md shadow-emerald-600/20 flex items-center gap-2 transition"
          >
            {#if isRunningQos}
              <span class="animate-spin text-xs">⏳</span>
              <span>Auditing QoS...</span>
            {:else}
              <span>📊 Audit Bufferbloat</span>
            {/if}
          </button>
        </div>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
        <!-- Speedometer Card -->
        <div class="p-6 rounded-2xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900/60 space-y-4">
          <span class="text-xs font-bold text-neutral-500 uppercase tracking-wider">Hasil Speed Test (CDN Edge)</span>
          {#if speedResult}
            <div class="grid grid-cols-2 gap-4 text-center">
              <div class="p-4 rounded-xl bg-sky-500/10 border border-sky-500/20">
                <span class="text-xs text-sky-600 dark:text-sky-400 font-medium">Download</span>
                <div class="text-3xl font-black text-sky-600 dark:text-sky-300 mt-1">{speedResult.downloadMbps}</div>
                <span class="text-[10px] text-neutral-400">Mbps</span>
              </div>
              <div class="p-4 rounded-xl bg-emerald-500/10 border border-emerald-500/20">
                <span class="text-xs text-emerald-600 dark:text-emerald-400 font-medium">Upload</span>
                <div class="text-3xl font-black text-emerald-600 dark:text-emerald-300 mt-1">{speedResult.uploadMbps}</div>
                <span class="text-[10px] text-neutral-400">Mbps</span>
              </div>
            </div>
            <div class="flex justify-between text-xs text-neutral-500 pt-3 border-t border-neutral-100 dark:border-neutral-800">
              <span>Ping: <strong class="text-neutral-900 dark:text-white font-mono">{speedResult.pingMs} ms</strong></span>
              <span>Jitter: <strong class="text-neutral-900 dark:text-white font-mono">{speedResult.jitterMs} ms</strong></span>
              <span>Target: <strong class="text-neutral-900 dark:text-white">{speedResult.serverTarget}</strong></span>
            </div>
          {:else}
            <div class="py-16 text-center text-neutral-400 text-xs">Belum ada data speed test. Tekan "Test Speedometer".</div>
          {/if}
        </div>

        <!-- QoS Bufferbloat Card -->
        <div class="p-6 rounded-2xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900/60 space-y-4">
          <span class="text-xs font-bold text-neutral-500 uppercase tracking-wider">Bufferbloat & QoS Grading</span>
          {#if qosResult}
            <div class="flex items-center gap-4">
              <div class="w-20 h-20 rounded-2xl flex items-center justify-center text-4xl font-black {qosResult.bufferbloatGrade === 'A+' || qosResult.bufferbloatGrade === 'A' ? 'bg-emerald-500/20 text-emerald-500 border border-emerald-500/30' : qosResult.bufferbloatGrade === 'B' || qosResult.bufferbloatGrade === 'C' ? 'bg-amber-500/20 text-amber-500 border border-amber-500/30' : 'bg-rose-500/20 text-rose-500 border border-rose-500/30'}">
                {qosResult.bufferbloatGrade}
              </div>
              <div class="space-y-1">
                <span class="text-xs font-semibold text-neutral-900 dark:text-white">Delta Lonjakan Latensi: +{qosResult.deltaLoadMs} ms</span>
                <p class="text-xs text-neutral-500">{qosResult.recommendation}</p>
              </div>
            </div>
            <div class="grid grid-cols-3 gap-2 pt-3 border-t border-neutral-100 dark:border-neutral-800 text-center font-mono text-xs">
              <div class="p-2.5 rounded-lg bg-neutral-50 dark:bg-neutral-800/50">
                <span class="text-[10px] text-neutral-400 block font-sans">Idle Ping</span>
                <strong>{qosResult.idleLatencyMs} ms</strong>
              </div>
              <div class="p-2.5 rounded-lg bg-neutral-50 dark:bg-neutral-800/50">
                <span class="text-[10px] text-neutral-400 block font-sans">Under Download</span>
                <strong>{qosResult.downloadLoadLatencyMs} ms</strong>
              </div>
              <div class="p-2.5 rounded-lg bg-neutral-50 dark:bg-neutral-800/50">
                <span class="text-[10px] text-neutral-400 block font-sans">Under Upload</span>
                <strong>{qosResult.uploadLoadLatencyMs} ms</strong>
              </div>
            </div>
          {:else}
            <div class="py-16 text-center text-neutral-400 text-xs">Belum ada audit QoS. Tekan "Audit Bufferbloat".</div>
          {/if}
        </div>
      </div>
    </div>
  {/if}

  <!-- TAB 3: SERVER BENCHMARK (PRO) -->
  {#if activeTab === 'benchmark'}
    <ProGate title="Network Diagnostics & Server Benchmark Pro" description="Benchmark multi-core CPU, memory bandwidth RAM, dan disk storage IOPS memerlukan lisensi CATerm Pro.">
      <div class="space-y-6">
        <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 p-4 rounded-2xl bg-white dark:bg-neutral-900/60 border border-neutral-200 dark:border-neutral-800">
          <div>
            <h3 class="text-sm font-bold text-neutral-900 dark:text-white">Hardware & Storage Benchmark</h3>
            <p class="text-xs text-neutral-500">Benchmark komputasi multi-core CPU, memory bandwidth RAM, dan disk write IOPS (4k/64k/1M direct IO).</p>
          </div>
          <button
            onclick={startBenchmark}
            disabled={isRunningBench || !selectedHostId}
            class="px-4 py-2 bg-amber-600 hover:bg-amber-500 disabled:opacity-50 text-white text-xs font-semibold rounded-xl flex items-center gap-2 transition shadow-md shadow-amber-600/20"
          >
            {#if isRunningBench}
              <span class="animate-spin text-xs">⏳</span>
              <span>Benchmarking Hardware...</span>
            {:else}
              <span>🔥 Jalankan Server Benchmark</span>
            {/if}
          </button>
        </div>

        {#if benchResult}
          <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
            <!-- CPU Card -->
            <div class="p-5 rounded-2xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900/60 space-y-3">
              <span class="text-xs font-bold text-neutral-500 uppercase">CPU Computing Core ({benchResult.cpuCores} Cores)</span>
              <div class="space-y-2 mt-2">
                <div class="flex justify-between items-center text-xs">
                  <span class="text-neutral-500">Single-Core Score:</span>
                  <strong class="font-mono text-base text-neutral-900 dark:text-white">{benchResult.cpuSingleScore} pts</strong>
                </div>
                <div class="flex justify-between items-center text-xs">
                  <span class="text-neutral-500">Multi-Core Score:</span>
                  <strong class="font-mono text-base text-amber-500">{benchResult.cpuMultiScore} pts</strong>
                </div>
              </div>
            </div>

            <!-- RAM Card -->
            <div class="p-5 rounded-2xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900/60 space-y-3">
              <span class="text-xs font-bold text-neutral-500 uppercase">Memory RAM Bandwidth</span>
              <div class="mt-2 text-center p-3 rounded-xl bg-sky-500/10 border border-sky-500/20">
                <div class="text-3xl font-black text-sky-600 dark:text-sky-400 font-mono">{benchResult.ramBandwidthMbps}</div>
                <span class="text-[10px] text-neutral-400">MB/s Direct Memory Throughput</span>
              </div>
            </div>

            <!-- Disk IOPS Card -->
            <div class="p-5 rounded-2xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900/60 space-y-3">
              <span class="text-xs font-bold text-neutral-500 uppercase">Disk Storage Throughput</span>
              <div class="space-y-1.5 text-xs font-mono">
                <div class="flex justify-between">
                  <span class="text-neutral-500">Random 4K IOPS:</span>
                  <strong>{benchResult.diskWriteIops4k} IOPS</strong>
                </div>
                <div class="flex justify-between">
                  <span class="text-neutral-500">Sequential 64K:</span>
                  <strong>{benchResult.diskWriteMbps64k} MB/s</strong>
                </div>
                <div class="flex justify-between">
                  <span class="text-neutral-500">Sequential 1M:</span>
                  <strong>{benchResult.diskWriteMbps1m} MB/s</strong>
                </div>
              </div>
            </div>
          </div>
        {:else}
          <div class="py-16 text-center text-neutral-400 text-xs bg-white dark:bg-neutral-900/40 rounded-2xl border border-neutral-200 dark:border-neutral-800">
            Belum ada benchmark dijalankan. Tekan "Jalankan Server Benchmark".
          </div>
        {/if}
      </div>
    </ProGate>
  {/if}

  <!-- TAB 4: MESH MATRIX & TOOLS LAB (PRO) -->
  {#if activeTab === 'mesh_tools'}
    <ProGate title="Mesh Latency Matrix & Diagnostics Lab Pro" description="Analisis RTT armada mesh server N x N, safe MTU finder, dan subnet sweeps memerlukan lisensi CATerm Pro.">
      <div class="space-y-6">
        <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 p-4 rounded-2xl bg-white dark:bg-neutral-900/60 border border-neutral-200 dark:border-neutral-800">
          <div>
            <h3 class="text-sm font-bold text-neutral-900 dark:text-white">Mesh Fleet Matrix, Subnet Sweeper & PMTUD</h3>
            <p class="text-xs text-neutral-500">RTT N x N antar armada server, pemindai CIDR lokal, penentu ukuran MTU aman, dan sertifikat SSL/TLS.</p>
          </div>
          <button
            onclick={startMeshMatrix}
            disabled={isRunningMesh || !selectedHostId}
            class="px-4 py-2 bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white text-xs font-semibold rounded-xl flex items-center gap-2 transition shadow-md shadow-indigo-600/20"
          >
            {#if isRunningMesh}
              <span class="animate-spin text-xs">⏳</span>
              <span>Probing Fleet...</span>
            {:else}
              <span>🌐 Probe Mesh Matrix Armada</span>
            {/if}
          </button>
        </div>

        <!-- Mesh Grid View -->
        {#if meshResult}
          <div class="p-5 rounded-2xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900/60 space-y-3">
            <span class="text-xs font-bold text-neutral-500 uppercase">Fleet Latency Grid ({meshResult.nodes.length} Target)</span>
            <div class="grid grid-cols-2 md:grid-cols-4 gap-3">
              {#each meshResult.nodes as node}
                <div class="p-3 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-800/50">
                  <div class="font-bold text-xs truncate text-neutral-900 dark:text-white">{node.targetLabel}</div>
                  <div class="text-[11px] text-neutral-400 truncate">{node.targetHost}</div>
                  <div class="mt-2 flex items-baseline justify-between">
                    <span class="font-mono text-sm font-bold {node.reachable ? 'text-emerald-500' : 'text-rose-500'}">
                      {node.reachable ? `${node.rttMs} ms` : 'UNREACHABLE'}
                    </span>
                    <span class="text-[10px] text-neutral-400">{node.packetLossPct}% loss</span>
                  </div>
                </div>
              {/each}
            </div>
          </div>
        {/if}

        <!-- Subnet Sweeper, PMTUD & TLS Row -->
        <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
          <!-- Subnet Sweeper -->
          <div class="p-5 rounded-2xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900/60 space-y-3">
            <span class="text-xs font-bold text-neutral-500 uppercase">Subnet CIDR Sweeper</span>
            <div class="flex gap-2">
              <input
                type="text"
                bind:value={subnetCidr}
                placeholder="192.168.1.0/24"
                class="flex-1 px-2.5 py-1.5 text-xs bg-neutral-50 dark:bg-neutral-800 border border-neutral-300 dark:border-neutral-700 rounded-lg text-neutral-900 dark:text-white font-mono"
              />
              <button
                onclick={startSubnetSweep}
                disabled={isRunningSubnet || !selectedHostId}
                class="px-3 py-1.5 bg-sky-600 text-white rounded-lg text-xs font-semibold hover:bg-sky-500 transition"
              >
                {isRunningSubnet ? '...' : 'Scan'}
              </button>
            </div>
            {#if subnetResult}
              <div class="text-xs text-neutral-500">Ditemukan <strong>{subnetResult.activeHosts}</strong> host aktif:</div>
              <div class="max-h-36 overflow-y-auto space-y-1 font-mono text-xs">
                {#each subnetResult.hosts as h}
                  <div class="p-1.5 rounded-lg bg-neutral-50 dark:bg-neutral-800 flex justify-between">
                    <span>{h.ip}</span>
                    <span class="text-[11px] text-sky-500">Ports: {h.openPorts.join(', ') || 'ICMP'}</span>
                  </div>
                {/each}
              </div>
            {/if}
          </div>

          <!-- PMTUD Probe -->
          <div class="p-5 rounded-2xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900/60 space-y-3">
            <span class="text-xs font-bold text-neutral-500 uppercase">PMTUD MTU Finder</span>
            <div class="flex gap-2">
              <input
                type="text"
                bind:value={pmtudTarget}
                placeholder="1.1.1.1"
                class="flex-1 px-2.5 py-1.5 text-xs bg-neutral-50 dark:bg-neutral-800 border border-neutral-300 dark:border-neutral-700 rounded-lg text-neutral-900 dark:text-white font-mono"
              />
              <button
                onclick={startPmtudProbe}
                disabled={isRunningPmtud || !selectedHostId}
                class="px-3 py-1.5 bg-sky-600 text-white rounded-lg text-xs font-semibold hover:bg-sky-500 transition"
              >
                {isRunningPmtud ? '...' : 'Find'}
              </button>
            </div>
            {#if pmtudResult}
              <div class="p-3 rounded-xl bg-sky-500/10 border border-sky-500/20 text-center">
                <span class="text-xs text-neutral-400 block">Optimal Safe MTU</span>
                <div class="text-2xl font-black text-sky-600 dark:text-sky-300 font-mono mt-1">{pmtudResult.optimalMtu}</div>
              </div>
              <p class="text-xs text-neutral-500">{pmtudResult.notes}</p>
            {/if}
          </div>

          <!-- TLS Certificate Auditor -->
          <div class="p-5 rounded-2xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900/60 space-y-3">
            <span class="text-xs font-bold text-neutral-500 uppercase">TLS Certificate Auditor</span>
            <div class="flex gap-2">
              <input
                type="text"
                bind:value={tlsHost}
                placeholder="domain.com"
                class="flex-1 px-2.5 py-1.5 text-xs bg-neutral-50 dark:bg-neutral-800 border border-neutral-300 dark:border-neutral-700 rounded-lg text-neutral-900 dark:text-white font-mono"
              />
              <button
                onclick={startTlsAudit}
                disabled={isRunningTls}
                class="px-3 py-1.5 bg-sky-600 text-white rounded-lg text-xs font-semibold hover:bg-sky-500 transition"
              >
                {isRunningTls ? '...' : 'Check'}
              </button>
            </div>
            {#if tlsResult}
              <div class="space-y-1.5 text-xs font-mono">
                <div class="flex justify-between">
                  <span class="text-neutral-500">Status:</span>
                  <strong class="{tlsResult.status === 'HEALTHY' ? 'text-emerald-500' : 'text-rose-500'}">{tlsResult.status}</strong>
                </div>
                <div class="flex justify-between">
                  <span class="text-neutral-500">Sisa Hari:</span>
                  <strong>{tlsResult.daysRemaining} hari</strong>
                </div>
                <div class="truncate text-[11px] text-neutral-400 pt-1">
                  Cipher: {tlsResult.cipherSuite}
                </div>
              </div>
            {/if}
          </div>
        </div>
      </div>
    </ProGate>
  {/if}
</div>
