<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { save } from '@tauri-apps/plugin-dialog';
  import { writeTextFile } from '@tauri-apps/plugin-fs';

  interface CommandLog {
    id: string;
    event_type: string;
    timestamp: number;
    host_id: string | null;
    details: string;
  }

  let logs: CommandLog[] = [];
  let filteredLogs: CommandLog[] = [];
  let hostFilter = '';
  let eventFilter = '';
  let dateFilter = '';
  let searchQuery = '';

  onMount(async () => {
    await fetchLogs();
  });

  async function fetchLogs() {
    try {
      logs = await invoke<CommandLog[]>('get_command_logs', {
        hostId: null,
        search: null
      });
      applyFilters();
    } catch (e) {
      console.error('Failed to fetch logs', e);
    }
  }

  function applyFilters() {
    filteredLogs = logs.filter(log => {
      if (hostFilter && log.host_id !== hostFilter) return false;
      if (eventFilter && log.event_type !== eventFilter) return false;
      
      if (dateFilter) {
        const logDate = new Date(log.timestamp).toISOString().split('T')[0];
        if (logDate !== dateFilter) return false;
      }
      
      if (searchQuery) {
        const query = searchQuery.toLowerCase();
        if (!log.details.toLowerCase().includes(query) && !log.event_type.toLowerCase().includes(query)) {
          return false;
        }
      }
      return true;
    });
  }

  $: {
    if (searchQuery !== null || hostFilter !== null || eventFilter !== null || dateFilter !== null) {
      applyFilters();
    }
  }

  async function exportLogs() {
    try {
      const csvContent = "ID,EventType,Timestamp,HostID,Details\n" + 
        filteredLogs.map(l => `"${l.id}","${l.event_type}","${new Date(l.timestamp).toISOString()}","${l.host_id || ''}","${l.details.replace(/"/g, '""')}"`).join("\n");
      
      const filePath = await save({
        filters: [{
          name: 'CSV',
          extensions: ['csv']
        }]
      });
      if (filePath) {
        await writeTextFile(filePath, csvContent);
        alert('Export successful!');
      }
    } catch (e) {
      console.error('Export failed', e);
      alert('Export failed.');
    }
  }

  function formatTime(ts: number) {
    return new Date(ts).toLocaleString();
  }
</script>

<div class="p-6 h-full flex flex-col bg-[#1E1E1E] text-white">
  <div class="flex justify-between items-center mb-6">
    <h1 class="text-2xl font-bold">Command Logs (Audit)</h1>
    <button on:click={exportLogs} class="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded shadow">Export CSV</button>
  </div>

  <div class="flex gap-4 mb-4">
    <input type="text" bind:value={searchQuery} placeholder="Search logs..." class="px-4 py-2 bg-[#2D2D2D] rounded border border-gray-700 flex-1" />
    <input type="text" bind:value={hostFilter} placeholder="Filter by Host ID" class="px-4 py-2 bg-[#2D2D2D] rounded border border-gray-700 w-48" />
    <select bind:value={eventFilter} class="px-4 py-2 bg-[#2D2D2D] rounded border border-gray-700 w-48">
      <option value="">All Events</option>
      <option value="PTY_COMMAND">PTY Command</option>
      <option value="TUNNEL_START">Tunnel Start</option>
      <option value="TUNNEL_STOP">Tunnel Stop</option>
      <option value="VAULT_LOCK">Vault Lock</option>
      <option value="VAULT_UNLOCK">Vault Unlock</option>
      <option value="KEY_DEPLOY">Key Deploy</option>
    </select>
    <input type="date" bind:value={dateFilter} class="px-4 py-2 bg-[#2D2D2D] rounded border border-gray-700 w-48" />
  </div>

  <div class="flex-1 overflow-auto bg-[#252526] rounded border border-gray-700">
    <table class="w-full text-left border-collapse">
      <thead class="sticky top-0 bg-[#2D2D2D] border-b border-gray-700">
        <tr>
          <th class="p-3">Time</th>
          <th class="p-3">Event</th>
          <th class="p-3">Host ID</th>
          <th class="p-3">Details</th>
        </tr>
      </thead>
      <tbody>
        {#each filteredLogs as log (log.id)}
          <tr class="border-b border-gray-700/50 hover:bg-[#2A2D2E]">
            <td class="p-3 whitespace-nowrap text-gray-400">{formatTime(log.timestamp)}</td>
            <td class="p-3">
              <span class="px-2 py-1 rounded text-xs font-mono bg-blue-500/20 text-blue-300">
                {log.event_type}
              </span>
            </td>
            <td class="p-3 text-gray-400">{log.host_id || '-'}</td>
            <td class="p-3 font-mono text-sm break-all">{log.details}</td>
          </tr>
        {/each}
        {#if filteredLogs.length === 0}
          <tr>
            <td colspan="4" class="p-8 text-center text-gray-500">No logs found</td>
          </tr>
        {/if}
      </tbody>
    </table>
  </div>
</div>
