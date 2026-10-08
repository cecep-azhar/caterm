<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import {
    runServerBenchmark,
    runNetworkSpeedTest,
    runQosAudit,
    runMeshLatencyMatrix,
    runPmtudProbe,
    runSubnetSweep,
    type ServerBenchmarkResult,
    type SpeedTestResult,
    type QosAuditResult,
    type MeshLatencyMatrix,
    type PmtudResult,
    type SubnetSweepResult
  } from '$lib/api/audit';
  import { listHosts, type HostRecord } from '$lib/api/hosts';
  import { showToast } from '$lib/stores/uiNotifications.svelte';
  import { errorText } from '$lib/errors';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import ProGate from '$lib/components/ProGate.svelte';

  let hosts = $state<HostRecord[]>([]);
  let selectedHostId = $state<string>('');
  let activeTab = $state<'speed_qos' | 'benchmark' | 'mesh' | 'network_tools'>('speed_qos');

  // Loading States
  let isRunningSpeed = $state(false);
  let isRunningQos = $state(false);
  let isRunningBench = $state(false);
  let isRunningMesh = $state(false);
  let isRunningSubnet = $state(false);
  let isRunningPmtud = $state(false);

  // Results State
  let speedResult = $state<SpeedTestResult | null>(null);
  let qosResult = $state<QosAuditResult | null>(null);
  let benchResult = $state<ServerBenchmarkResult | null>(null);
  let meshResult = $state<MeshLatencyMatrix | null>(null);
  let subnetResult = $state<SubnetSweepResult | null>(null);
  let pmtudResult = $state<PmtudResult | null>(null);

  // Tools Inputs
  let subnetCidr = $state('192.168.1.0/24');
  let pmtudTarget = $state('1.1.1.1');

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

  async function startServerBenchmark() {
    if (!selectedHostId) return;
    isRunningBench = true;
    try {
      benchResult = await runServerBenchmark(selectedHostId);
      showToast(`Benchmark selesai! RAM: ${benchResult.ramBandwidthMbps} MB/s, Disk: ${benchResult.diskWriteMbps} MB/s`, 'success');
    } catch (e) {
      showToast(`Benchmark gagal: ${errorText(e)}`, 'error');
    } finally {
      isRunningBench = false;
    }
  }

  async function startMeshMatrix() {
    if (hosts.length === 0) {
      showToast('Perlu minimal 1 host untuk menguji mesh matrix', 'error');
      return;
    }
    isRunningMesh = true;
    try {
      const hostIds = hosts.map((h) => h.id);
      meshResult = await runMeshLatencyMatrix(hostIds);
      showToast('Mesh Latency Matrix selesai dihitung!', 'success');
    } catch (e) {
      showToast(`Mesh Matrix gagal: ${errorText(e)}`, 'error');
    } finally {
      isRunningMesh = false;
    }
  }

  async function startSubnetSweep() {
    if (!selectedHostId) return;
    isRunningSubnet = true;
    try {
      subnetResult = await runSubnetSweep(selectedHostId, subnetCidr);
      showToast(`Subnet Sweep selesai: ${subnetResult.activeHosts} host aktif`, 'success');
    } catch (e) {
      showToast(`Subnet Sweep gagal: ${errorText(e)}`, 'error');
    } finally {
      isRunningSubnet = false;
    }
  }

  async function startPmtudProbe() {
    if (!selectedHostId) return;
    isRunningPmtud = true;
    try {
      pmtudResult = await runPmtudProbe(selectedHostId, pmtudTarget);
      showToast(`PMTUD Selesai! Optimal MTU: ${pmtudResult.optimalMtu}`, 'success');
    } catch (e) {
      showToast(`PMTUD gagal: ${errorText(e)}`, 'error');
    } finally {
      isRunningPmtud = false;
    }
  }
</script>

<div class="p-6 max-w-7xl mx-auto space-y-6">
  <!-- HEADER -->
  <div class="flex flex-col md:flex-row md:items-center justify-between gap-4">
    <PageHeader
      title="DevOps & Performance Lab"
      subtitle="Benchmark komputasi server, throughput kecepatan jaringan & bufferbloat QoS, serta mesh latency matrix multi-node."
    />
    <div class="flex items-center gap-3">
      <!-- Host Selector -->
      <div class="flex items-center gap-2 bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-xl px-3 py-1.5 shadow-xs">
        <label for="host-select-devops" class="text-xs font-semibold text-neutral-500 uppercase">Target Host:</label>
        <select
          id="host-select-devops"
          bind:value={selectedHostId}
          class="bg-transparent text-xs font-bold text-neutral-900 dark:text-white focus:outline-none cursor-pointer"
        >
          <option value="local">Localhost (Mesin Ini)</option>
          {#each hosts as h}
            <option value={h.id}>{h.label} ({h.address})</option>
          {/each}
        </select>
      </div>
    </div>
  </div>

  <!-- NAVIGATION TABS -->
  <div class="flex items-center gap-2 border-b border-neutral-200 dark:border-neutral-800 pb-2 overflow-x-auto text-xs font-semibold">
    <button
      onclick={() => (activeTab = 'speed_qos')}
      class="px-4 py-2 rounded-xl transition flex items-center gap-2 whitespace-nowrap {activeTab === 'speed_qos' ? 'bg-white dark:bg-neutral-800 text-sky-600 dark:text-sky-400 shadow-sm border border-neutral-200/50 dark:border-neutral-700' : 'text-neutral-600 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white'}"
    >
      <span>⚡ Throughput & QoS Bufferbloat</span>
    </button>
    <button
      onclick={() => (activeTab = 'benchmark')}
      class="px-4 py-2 rounded-xl transition flex items-center gap-2 whitespace-nowrap {activeTab === 'benchmark' ? 'bg-white dark:bg-neutral-800 text-amber-600 dark:text-amber-400 shadow-sm border border-neutral-200/50 dark:border-neutral-700' : 'text-neutral-600 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white'}"
    >
      <span>🚀 Server Hardware Benchmark</span>
    </button>
    <button
      onclick={() => (activeTab = 'mesh')}
      class="px-4 py-2 rounded-xl transition flex items-center gap-2 whitespace-nowrap {activeTab === 'mesh' ? 'bg-white dark:bg-neutral-800 text-indigo-600 dark:text-indigo-400 shadow-sm border border-neutral-200/50 dark:border-neutral-700' : 'text-neutral-600 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white'}"
    >
      <span>🕸️ Fleet Mesh Latency Matrix (Pro)</span>
      <span class="text-[9px] px-1.5 py-0.5 rounded bg-indigo-500/20 text-indigo-500 uppercase font-black">PRO</span>
    </button>
    <button
      onclick={() => (activeTab = 'network_tools')}
      class="px-4 py-2 rounded-xl transition flex items-center gap-2 whitespace-nowrap {activeTab === 'network_tools' ? 'bg-white dark:bg-neutral-800 text-emerald-600 dark:text-emerald-400 shadow-sm border border-neutral-200/50 dark:border-neutral-700' : 'text-neutral-600 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white'}"
    >
      <span>🔍 PMTUD & Subnet Tools</span>
    </button>
  </div>

  <!-- TAB 1: SPEED TEST & QOS -->
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
            {isRunningSpeed ? 'Testing...' : '⚡ Jalankan Speed Test'}
          </button>
          <button
            onclick={startQosAudit}
            disabled={isRunningQos || !selectedHostId}
            class="px-4 py-2 bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white text-xs font-semibold rounded-xl shadow-md shadow-indigo-600/20 flex items-center gap-2 transition"
          >
            {isRunningQos ? 'Auditing...' : '📊 Audit Bufferbloat QoS'}
          </button>
        </div>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <!-- Speed Results -->
        <div class="p-6 rounded-2xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900/60 space-y-4">
          <h4 class="text-xs font-bold uppercase tracking-wider text-neutral-500">Bandwidth Throughput</h4>
          {#if speedResult}
            <div class="grid grid-cols-2 gap-4">
              <div class="p-4 rounded-xl bg-sky-500/10 border border-sky-500/20">
                <span class="text-xs text-neutral-400">Download</span>
                <div class="text-3xl font-black text-sky-600 dark:text-sky-400 mt-1">{speedResult.downloadMbps} <span class="text-xs font-normal">Mbps</span></div>
              </div>
              <div class="p-4 rounded-xl bg-indigo-500/10 border border-indigo-500/20">
                <span class="text-xs text-neutral-400">Upload</span>
                <div class="text-3xl font-black text-indigo-600 dark:text-indigo-400 mt-1">{speedResult.uploadMbps} <span class="text-xs font-normal">Mbps</span></div>
              </div>
            </div>
            <div class="text-xs text-neutral-500 font-mono space-y-1">
              <div>Latency: <strong>{speedResult.latencyMs} ms</strong> (Jitter: {speedResult.jitterMs} ms)</div>
              <div>Server: {speedResult.serverName} ({speedResult.serverLocation})</div>
            </div>
          {:else}
            <div class="py-12 text-center text-neutral-400 text-xs">Tekan "Jalankan Speed Test" untuk menguji bandwidth target.</div>
          {/if}
        </div>

        <!-- QoS Bufferbloat -->
        <div class="p-6 rounded-2xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900/60 space-y-4">
          <h4 class="text-xs font-bold uppercase tracking-wider text-neutral-500">Bufferbloat QoS Rating</h4>
          {#if qosResult}
            <div class="flex items-center gap-4 p-4 rounded-xl bg-neutral-50 dark:bg-neutral-800/50 border border-neutral-200 dark:border-neutral-700">
              <div class="w-16 h-16 rounded-2xl flex items-center justify-center font-black text-2xl {qosResult.bufferbloatGrade.startsWith('A') ? 'bg-emerald-500/20 text-emerald-500' : qosResult.bufferbloatGrade.startsWith('B') ? 'bg-sky-500/20 text-sky-500' : 'bg-rose-500/20 text-rose-500'}">
                {qosResult.bufferbloatGrade}
              </div>
              <div>
                <span class="text-xs text-neutral-400">Degradasi Latensi Saat Beban Penuh:</span>
                <div class="text-sm font-bold text-neutral-900 dark:text-white mt-0.5">+{qosResult.bloatLatencyDeltaMs} ms delay</div>
                <p class="text-xs text-neutral-500 mt-1">{qosResult.recommendation}</p>
              </div>
            </div>
            <div class="grid grid-cols-2 gap-2 text-xs font-mono">
              <div class="p-2.5 rounded-lg bg-neutral-100 dark:bg-neutral-800/40">Idle Ping: <strong>{qosResult.idleLatencyMs} ms</strong></div>
              <div class="p-2.5 rounded-lg bg-neutral-100 dark:bg-neutral-800/40">Loaded Ping: <strong>{qosResult.loadedLatencyMs} ms</strong></div>
            </div>
          {:else}
            <div class="py-12 text-center text-neutral-400 text-xs">Tekan "Audit Bufferbloat QoS" untuk mendiagnosis bufferbloat antrian router.</div>
          {/if}
        </div>
      </div>
    </div>
  {/if}

  <!-- TAB 2: SERVER HARDWARE BENCHMARK -->
  {#if activeTab === 'benchmark'}
    <div class="space-y-6">
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 p-4 rounded-2xl bg-white dark:bg-neutral-900/60 border border-neutral-200 dark:border-neutral-800">
        <div>
          <h3 class="text-sm font-bold text-neutral-900 dark:text-white">Server Hardware Benchmark Suite</h3>
          <p class="text-xs text-neutral-500">Benchmark komputasi multi-core CPU, throughput baca/tulis RAM, dan sequential & random I/O write speed.</p>
        </div>
        <button
          onclick={startServerBenchmark}
          disabled={isRunningBench || !selectedHostId}
          class="px-4 py-2 bg-amber-600 hover:bg-amber-500 disabled:opacity-50 text-white text-xs font-semibold rounded-xl shadow-md shadow-amber-600/20 flex items-center gap-2 transition"
        >
          {isRunningBench ? 'Running Benchmark...' : '🚀 Jalankan Server Benchmark'}
        </button>
      </div>

      {#if benchResult}
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
          <div class="p-5 rounded-2xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900/60 shadow-xs">
            <span class="text-xs font-semibold text-neutral-500 uppercase">CPU Single-Core</span>
            <div class="text-2xl font-black text-neutral-900 dark:text-white mt-1">{benchResult.cpuSingleScore} <span class="text-xs font-normal text-neutral-400">pts</span></div>
          </div>
          <div class="p-5 rounded-2xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900/60 shadow-xs">
            <span class="text-xs font-semibold text-neutral-500 uppercase">CPU Multi-Core</span>
            <div class="text-2xl font-black text-amber-500 mt-1">{benchResult.cpuMultiScore} <span class="text-xs font-normal text-neutral-400">pts</span></div>
          </div>
          <div class="p-5 rounded-2xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900/60 shadow-xs">
            <span class="text-xs font-semibold text-neutral-500 uppercase">RAM Bandwidth</span>
            <div class="text-2xl font-black text-sky-500 mt-1">{benchResult.ramBandwidthMbps} <span class="text-xs font-normal text-neutral-400">MB/s</span></div>
          </div>
          <div class="p-5 rounded-2xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900/60 shadow-xs">
            <span class="text-xs font-semibold text-neutral-500 uppercase">Disk I/O Write</span>
            <div class="text-2xl font-black text-emerald-500 mt-1">{benchResult.diskWriteMbps} <span class="text-xs font-normal text-neutral-400">MB/s</span></div>
          </div>
        </div>

        <div class="p-5 rounded-2xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900/60 space-y-3">
          <h4 class="text-xs font-bold uppercase tracking-wider text-neutral-500">Hardware Profile Target</h4>
          <div class="grid grid-cols-1 md:grid-cols-3 gap-3 text-xs font-mono">
            <div class="p-3 rounded-xl bg-neutral-50 dark:bg-neutral-800/50">CPU Model: <strong>{benchResult.cpuModel}</strong></div>
            <div class="p-3 rounded-xl bg-neutral-50 dark:bg-neutral-800/50">Core Count: <strong>{benchResult.cpuCores} Cores</strong></div>
            <div class="p-3 rounded-xl bg-neutral-50 dark:bg-neutral-800/50">Disk IOPS Random: <strong>{benchResult.diskIops} IOPS</strong></div>
          </div>
        </div>
      {:else}
        <div class="py-16 text-center text-neutral-400 text-xs bg-white dark:bg-neutral-900/40 rounded-2xl border border-neutral-200 dark:border-neutral-800">
          Belum ada benchmark dijalankan. Tekan "Jalankan Server Benchmark".
        </div>
      {/if}
    </div>
  {/if}

  <!-- TAB 3: MESH LATENCY MATRIX (PRO) -->
  {#if activeTab === 'mesh'}
    <div class="space-y-6">
      <ProGate title="Fleet Mesh Latency Matrix Pro" description="Uji interkoneksi full-mesh antar server armada DevOps membutuhkan lisensi CATerm Pro.">
        <div class="p-5 rounded-2xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900/60 space-y-4">
          <div class="flex items-center justify-between">
            <div>
              <h4 class="text-xs font-bold uppercase tracking-wider text-neutral-900 dark:text-white">Multi-Host Mesh Latency Grid</h4>
              <p class="text-xs text-neutral-500">Matriks latensi antar node armada (ThinkPad X1, DELL YPC, Hostinger VPS, Lenovo XPC).</p>
            </div>
            <button
              onclick={startMeshMatrix}
              disabled={isRunningMesh}
              class="px-4 py-2 bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white text-xs font-semibold rounded-xl transition"
            >
              {isRunningMesh ? 'Computing Matrix...' : 'Compute Mesh Matrix'}
            </button>
          </div>

          {#if meshResult}
            <div class="overflow-x-auto">
              <table class="w-full text-xs text-left border-collapse font-mono">
                <thead>
                  <tr class="border-b border-neutral-200 dark:border-neutral-800 text-neutral-500">
                    <th class="p-2">From \ To</th>
                    {#each meshResult.nodes as node}
                      <th class="p-2">{node.name}</th>
                    {/each}
                  </tr>
                </thead>
                <tbody>
                  {#each meshResult.nodes as srcNode, i}
                    <tr class="border-b border-neutral-100 dark:border-neutral-800/60">
                      <td class="p-2 font-bold text-neutral-900 dark:text-white">{srcNode.name}</td>
                      {#each meshResult.nodes as dstNode, j}
                        <td class="p-2">
                          {#if i === j}
                            <span class="text-neutral-400">-</span>
                          {:else}
                            {@const latency = meshResult.matrix[i]?.[j]}
                            <span class="font-bold {latency < 30 ? 'text-emerald-500' : latency < 100 ? 'text-amber-500' : 'text-rose-500'}">
                              {latency ? `${latency.toFixed(1)} ms` : 'Timeout'}
                            </span>
                          {/if}
                        </td>
                      {/each}
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          {:else}
            <div class="py-8 text-center text-neutral-400 text-xs">Tekan "Compute Mesh Matrix" untuk menghitung matriks latensi antar host.</div>
          {/if}
        </div>
      </ProGate>
    </div>
  {/if}

  <!-- TAB 4: PMTUD & SUBNET TOOLS -->
  {#if activeTab === 'network_tools'}
    <div class="space-y-6">
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <!-- Subnet CIDR Sweeper -->
        <div class="p-5 rounded-2xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900/60 space-y-3">
          <div>
            <span class="text-xs font-bold text-neutral-900 dark:text-white uppercase tracking-wider">Subnet CIDR Sweeper</span>
            <p class="text-xs text-neutral-500 mt-0.5">Pemindaian host aktif dan port umum dalam blok CIDR jaringan lokal.</p>
          </div>
          <div class="flex gap-2">
            <input
              type="text"
              bind:value={subnetCidr}
              placeholder="192.168.1.0/24"
              class="flex-1 px-3 py-2 text-xs bg-neutral-50 dark:bg-neutral-800 border border-neutral-300 dark:border-neutral-700 rounded-lg text-neutral-900 dark:text-white font-mono"
            />
            <button
              onclick={startSubnetSweep}
              disabled={isRunningSubnet || !selectedHostId}
              class="px-4 py-2 bg-sky-600 text-white rounded-lg text-xs font-semibold hover:bg-sky-500 disabled:opacity-50 transition"
            >
              {isRunningSubnet ? 'Scanning...' : 'Sweep CIDR'}
            </button>
          </div>
          {#if subnetResult}
            <div class="text-xs text-neutral-500">Ditemukan <strong>{subnetResult.activeHosts}</strong> host aktif:</div>
            <div class="max-h-48 overflow-y-auto space-y-1 font-mono text-xs">
              {#each subnetResult.hosts as h}
                <div class="p-2 rounded-lg bg-neutral-50 dark:bg-neutral-800 flex justify-between border border-neutral-100 dark:border-neutral-700/60">
                  <span class="font-bold">{h.ip}</span>
                  <span class="text-[11px] text-sky-500">Open: {h.openPorts.join(', ') || 'ICMP Only'}</span>
                </div>
              {/each}
            </div>
          {/if}
        </div>

        <!-- PMTUD Probe -->
        <div class="p-5 rounded-2xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900/60 space-y-3">
          <div>
            <span class="text-xs font-bold text-neutral-900 dark:text-white uppercase tracking-wider">Path MTU Discovery (PMTUD)</span>
            <p class="text-xs text-neutral-500 mt-0.5">Uji batas ukuran paket maksimal tanpa fragmentasi untuk mencegah paket drop.</p>
          </div>
          <div class="flex gap-2">
            <input
              type="text"
              bind:value={pmtudTarget}
              placeholder="1.1.1.1"
              class="flex-1 px-3 py-2 text-xs bg-neutral-50 dark:bg-neutral-800 border border-neutral-300 dark:border-neutral-700 rounded-lg text-neutral-900 dark:text-white font-mono"
            />
            <button
              onclick={startPmtudProbe}
              disabled={isRunningPmtud || !selectedHostId}
              class="px-4 py-2 bg-sky-600 text-white rounded-lg text-xs font-semibold hover:bg-sky-500 disabled:opacity-50 transition"
            >
              {isRunningPmtud ? 'Probing...' : 'Probe MTU'}
            </button>
          </div>
          {#if pmtudResult}
            <div class="p-4 rounded-xl bg-sky-500/10 border border-sky-500/20 text-center space-y-1">
              <span class="text-xs text-neutral-400 block uppercase">Optimal Safe MTU</span>
              <div class="text-3xl font-black text-sky-600 dark:text-sky-300 font-mono">{pmtudResult.optimalMtu}</div>
              <p class="text-xs text-neutral-500">{pmtudResult.notes}</p>
            </div>
          {/if}
        </div>
      </div>
    </div>
  {/if}
</div>
