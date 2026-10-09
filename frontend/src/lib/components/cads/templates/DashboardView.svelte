<script lang="ts">
  import Card from '$lib/components/ui/Card.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Badge from '$lib/components/ui/Badge.svelte';
  import Table, { type Column } from '$lib/components/ui/Table.svelte';
  import Icon from '$lib/components/ui/Icon.svelte';

  interface Metric {
    label: string;
    value: string;
    change: string;
    trend: 'up' | 'down' | 'neutral';
    icon: 'server' | 'cpu' | 'zap' | 'shield';
  }

  interface ServerRow {
    id: string;
    name: string;
    ip: string;
    region: string;
    cpu: string;
    mem: string;
    status: 'online' | 'warning' | 'offline';
  }

  const metrics: Metric[] = [
    { label: 'Active Sessions', value: '18', change: '+3 now', trend: 'up', icon: 'server' },
    { label: 'Fleet CPU Avg', value: '34.2%', change: '-2.1%', trend: 'down', icon: 'cpu' },
    { label: 'Memory Allocated', value: '48.6 GB', change: '64%', trend: 'neutral', icon: 'zap' },
    { label: 'SSH Zero-Trust Gates', value: '100%', change: 'Active', trend: 'up', icon: 'shield' },
  ];

  const columns: Column<ServerRow>[] = [
    { key: 'name', label: 'Host Node', sortable: true },
    { key: 'ip', label: 'IP Address / Domain', sortable: true },
    { key: 'region', label: 'Region', sortable: true },
    { key: 'cpu', label: 'CPU Load', sortable: true, align: 'right' },
    { key: 'mem', label: 'Memory', sortable: true, align: 'right' },
    { key: 'status', label: 'Health', sortable: true, align: 'center' },
  ];

  let serverData = $state<ServerRow[]>([
    { id: '1', name: 'prod-api-cluster-01', ip: '104.21.48.12', region: 'SG-SIN', cpu: '28%', mem: '4.2/8 GB', status: 'online' },
    { id: '2', name: 'db-primary-postgresql', ip: '104.21.48.15', region: 'SG-SIN', cpu: '68%', mem: '28/32 GB', status: 'warning' },
    { id: '3', name: 'redis-cache-tier-01', ip: '10.0.4.101', region: 'US-EAST', cpu: '14%', mem: '12/16 GB', status: 'online' },
    { id: '4', name: 'edge-proxy-cloudflared', ip: '198.51.100.4', region: 'EU-FRA', cpu: '9%', mem: '1.2/4 GB', status: 'online' },
    { id: '5', name: 'backup-vols-secondary', ip: '10.0.8.200', region: 'AP-TYO', cpu: '0%', mem: '0/8 GB', status: 'offline' },
  ]);

  let selectedServer = $state<ServerRow | null>(serverData[0]);
</script>

<div class="space-y-6 font-sans text-[#EDEDED]">
  <!-- Top Quick Actions & Title -->
  <div class="flex flex-wrap items-center justify-between gap-4">
    <div>
      <h2 class="text-lg font-bold text-white tracking-tight">Cloud Fleet Telemetry</h2>
      <p class="text-xs text-neutral-400">Real-time overview of active SSH nodes and AI co-pilot status</p>
    </div>
    <div class="flex items-center gap-2">
      <Button variant="outline" size="sm">
        <Icon name="refresh" size={12} />
        Sync Metrics
      </Button>
      <Button variant="brand" size="sm">
        <Icon name="terminal" size={12} />
        Launch Global Terminal
      </Button>
    </div>
  </div>

  <!-- Metric Telemetry Cards Grid -->
  <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
    {#each metrics as m}
      <div class="p-4 rounded-xl bg-[#121217] border border-[#272732] flex items-center justify-between shadow-xs">
        <div class="space-y-1">
          <span class="text-[11px] font-medium text-neutral-400 uppercase tracking-wider">{m.label}</span>
          <div class="text-2xl font-bold font-mono text-white">{m.value}</div>
          <div class="text-[10px] flex items-center gap-1 {m.trend === 'up' ? 'text-emerald-400' : m.trend === 'down' ? 'text-sky-400' : 'text-neutral-400'}">
            <span>{m.change}</span>
          </div>
        </div>
        <div class="w-10 h-10 rounded-lg bg-[#18181F] border border-[#272732] flex items-center justify-center text-[var(--ca-brand)]">
          <Icon name={m.icon} size={18} />
        </div>
      </div>
    {/each}
  </div>

  <!-- Split View: Table Grid + Node Inspector -->
  <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
    <!-- Server DataGrid (2 cols) -->
    <div class="lg:col-span-2 space-y-3">
      <div class="flex items-center justify-between">
        <h3 class="text-xs font-bold uppercase tracking-wider text-neutral-400">Provisioned Servers ({serverData.length})</h3>
        <span class="text-[11px] text-neutral-500">Click row to inspect</span>
      </div>

      <Table
        {columns}
        data={serverData}
        striped
        pageSize={5}
        onRowClick={(row) => (selectedServer = row)}
      >
        {#snippet cell({ column, value })}
          {#if column.key === 'name'}
            <span class="font-semibold text-white font-mono text-xs">{value}</span>
          {:else if column.key === 'ip'}
            <span class="font-mono text-neutral-400 text-xs">{value}</span>
          {:else if column.key === 'status'}
            <Badge
              variant={value === 'online' ? 'success' : value === 'warning' ? 'warning' : 'danger'}
              size="sm"
            >
              {value}
            </Badge>
          {:else}
            <span class="font-mono text-xs">{value}</span>
          {/if}
        {/snippet}

        {#snippet actions({ item })}
          <div class="flex items-center justify-end gap-1">
            <button
              type="button"
              onclick={() => (selectedServer = item)}
              class="p-1 rounded text-neutral-400 hover:text-white hover:bg-neutral-800 transition cursor-pointer"
              title="Inspect"
            >
              <Icon name="search" size={12} />
            </button>
            <button
              type="button"
              class="p-1 rounded text-[var(--ca-brand)] hover:bg-[var(--ca-brand)]/20 transition cursor-pointer"
              title="Connect SSH"
            >
              <Icon name="terminal" size={12} />
            </button>
          </div>
        {/snippet}
      </Table>
    </div>

    <!-- Inspector Sidecard (1 col) -->
    <div class="space-y-3">
      <h3 class="text-xs font-bold uppercase tracking-wider text-neutral-400">Node Inspector</h3>
      {#if selectedServer}
        <Card title={selectedServer.name} description="Dedicated cloud node telemetry details">
          <div class="space-y-3 text-xs">
            <div class="flex justify-between py-1 border-b border-[#272732]">
              <span class="text-neutral-400">Status</span>
              <Badge variant={selectedServer.status === 'online' ? 'success' : selectedServer.status === 'warning' ? 'warning' : 'danger'}>
                {selectedServer.status.toUpperCase()}
              </Badge>
            </div>
            <div class="flex justify-between py-1 border-b border-[#272732]">
              <span class="text-neutral-400">Public IP</span>
              <span class="font-mono text-white">{selectedServer.ip}</span>
            </div>
            <div class="flex justify-between py-1 border-b border-[#272732]">
              <span class="text-neutral-400">Region Zone</span>
              <span class="text-white">{selectedServer.region}</span>
            </div>
            <div class="flex justify-between py-1 border-b border-[#272732]">
              <span class="text-neutral-400">Live CPU Load</span>
              <span class="font-mono text-white">{selectedServer.cpu}</span>
            </div>
            <div class="flex justify-between py-1 border-b border-[#272732]">
              <span class="text-neutral-400">Memory Usage</span>
              <span class="font-mono text-white">{selectedServer.mem}</span>
            </div>

            <div class="pt-3 flex gap-2">
              <Button variant="brand" size="sm" class="flex-1 justify-center">
                SSH Connect
              </Button>
              <Button variant="outline" size="sm" class="flex-1 justify-center">
                Reboot Node
              </Button>
            </div>
          </div>
        </Card>
      {:else}
        <div class="p-8 rounded-xl border border-dashed border-[#272732] text-center text-xs text-neutral-500">
          Select a server row from the table to view real-time diagnostics.
        </div>
      {/if}
    </div>
  </div>
</div>
