<script lang="ts">
  import { onMount } from 'svelte';
  import { listInvestigations, saveInvestigation, deleteInvestigation, type InvestigationRecord } from '$lib/api/investigations';
  import { listHosts, type HostRecord } from '$lib/api/hosts';

  let isAddModalOpen = $state(false);
  let backendAvailable = $state(true);
  let investigations = $state<InvestigationRecord[]>([]);
  let availableHosts = $state<HostRecord[]>([]);
  let editingId = $state<string | null>(null);

  let newInvestigation = $state({
    title: "",
    host_id: "" as string | null,
    status: "OPEN",
    notes: "",
    evidence: "[]"
  });

  onMount(async () => {
    try {
      [investigations, availableHosts] = await Promise.all([
        listInvestigations(),
        listHosts()
      ]);
    } catch {
      backendAvailable = false;
    }
  });

  function hostLabel(id: string | null) {
    if (!id) return "Global";
    return availableHosts.find((h) => h.id === id)?.label ?? id;
  }

  function openAddModal() {
    editingId = null;
    newInvestigation = { title: "", host_id: "", status: "OPEN", notes: "", evidence: "[]" };
    isAddModalOpen = true;
  }

  function openEditModal(inv: InvestigationRecord) {
    editingId = inv.id;
    newInvestigation = {
      title: inv.title,
      host_id: inv.host_id,
      status: inv.status,
      notes: inv.notes,
      evidence: inv.evidence
    };
    isAddModalOpen = true;
  }

  async function createInvestigation(e: Event) {
    e.preventDefault();
    if (!newInvestigation.title) return;

    try {
      const saved = await saveInvestigation({
        id: editingId ?? undefined,
        title: newInvestigation.title,
        host_id: newInvestigation.host_id || null,
        status: newInvestigation.status,
        notes: newInvestigation.notes,
        evidence: newInvestigation.evidence
      });
      investigations = editingId
        ? investigations.map((i) => (i.id === saved.id ? saved : i))
        : [saved, ...investigations];
    } catch {
      backendAvailable = false;
    }

    editingId = null;
    isAddModalOpen = false;
  }

  async function removeInvestigation(id: string) {
    try {
      await deleteInvestigation(id);
      investigations = investigations.filter((i) => i.id !== id);
    } catch {
      backendAvailable = false;
    }
  }

  function formatDate(ts: number) {
    return new Date(ts).toLocaleString();
  }
</script>

<div class="max-w-5xl mx-auto space-y-6">
  <div class="flex justify-between items-center">
    <div>
      <h1 class="text-2xl font-bold text-white tracking-tight">Investigations</h1>
      <p class="text-neutral-400 text-sm mt-1">Review saved security incidents, timelines, and audit evidence.</p>
      {#if !backendAvailable}
        <p class="text-amber-500 text-xs mt-1">Tauri backend not detected — changes won't be saved to disk.</p>
      {/if}
    </div>
    <button
      onclick={openAddModal}
      class="px-4 py-2 bg-sky-600 hover:bg-sky-500 text-white font-medium text-sm rounded-md transition-colors flex items-center gap-2">
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"></path></svg>
      New Investigation
    </button>
  </div>

  <div class="grid grid-cols-1 gap-4">
    {#each investigations as inv}
      <div class="bg-neutral-900 border border-neutral-800 rounded-lg p-5 flex flex-col justify-between hover:border-neutral-700 transition-colors">
        <div>
          <div class="flex justify-between items-start">
            <h3 class="font-semibold text-white text-lg">{inv.title}</h3>
            <span class="px-2 py-1 text-xs rounded border {inv.status === 'OPEN' ? 'border-amber-500/50 text-amber-500' : 'border-green-500/50 text-green-500'}">
              {inv.status}
            </span>
          </div>
          <p class="text-neutral-400 text-sm mt-1">Host: {hostLabel(inv.host_id)}</p>
          <p class="text-neutral-500 text-xs mt-1">Opened: {formatDate(inv.created_at)}</p>
          {#if inv.notes}
            <div class="mt-3 p-3 bg-neutral-950 border border-neutral-800 rounded text-neutral-300 text-sm whitespace-pre-wrap">{inv.notes}</div>
          {/if}
          <div class="mt-3 flex gap-2">
            <span class="text-xs text-neutral-400 border border-neutral-700 px-2 py-1 rounded">Evidence attached: {inv.evidence !== '[]' ? 'Yes' : 'None'}</span>
          </div>
        </div>
        <div class="mt-4 pt-3 border-t border-neutral-800/80 flex justify-end gap-2">
          <button onclick={() => openEditModal(inv)} class="px-3 py-1 bg-neutral-800 hover:bg-sky-600 hover:text-white rounded text-xs text-neutral-300 transition-colors">Edit</button>
          <button onclick={() => removeInvestigation(inv.id)} class="px-3 py-1 bg-neutral-800 hover:bg-red-600 hover:text-white rounded text-xs text-neutral-300 transition-colors">Delete</button>
        </div>
      </div>
    {/each}
    {#if investigations.length === 0}
      <div class="text-center py-10 text-neutral-500 border border-dashed border-neutral-800 rounded-lg">
        No investigations found.
      </div>
    {/if}
  </div>
</div>

{#if isAddModalOpen}
  <div class="fixed inset-0 bg-black/70 flex items-center justify-center p-4 z-50">
    <div class="bg-neutral-900 border border-neutral-800 rounded-lg p-6 max-w-lg w-full space-y-4 max-h-[90vh] overflow-y-auto">
      <h3 class="text-xl font-bold text-white">{editingId ? 'Edit Investigation' : 'New Investigation'}</h3>
      
      <form onsubmit={createInvestigation} class="space-y-4">
        <div>
          <label for="inv-title" class="block text-xs font-medium text-neutral-400 uppercase mb-1">Incident Title</label>
          <input id="inv-title" bind:value={newInvestigation.title} required placeholder="e.g. Unauthorized login attempt" class="w-full px-3 py-2 bg-neutral-950 border border-neutral-800 rounded text-sm text-white focus:outline-none focus:border-sky-500" />
        </div>

        <div>
          <label for="inv-host" class="block text-xs font-medium text-neutral-400 uppercase mb-1">Target Host</label>
          <select id="inv-host" bind:value={newInvestigation.host_id} class="w-full px-3 py-2 bg-neutral-950 border border-neutral-800 rounded text-sm text-white focus:outline-none focus:border-sky-500">
            <option value="">-- Global / Multiple Hosts --</option>
            {#each availableHosts as h}
              <option value={h.id}>{h.label}</option>
            {/each}
          </select>
        </div>

        <div>
          <label for="inv-status" class="block text-xs font-medium text-neutral-400 uppercase mb-1">Status</label>
          <select id="inv-status" bind:value={newInvestigation.status} class="w-full px-3 py-2 bg-neutral-950 border border-neutral-800 rounded text-sm text-white focus:outline-none focus:border-sky-500">
            <option value="OPEN">Open</option>
            <option value="IN_PROGRESS">In Progress</option>
            <option value="CLOSED">Closed</option>
          </select>
        </div>

        <div>
          <label for="inv-notes" class="block text-xs font-medium text-neutral-400 uppercase mb-1">Manual Notes</label>
          <textarea id="inv-notes" bind:value={newInvestigation.notes} placeholder="Analyst notes, timeline details, etc." rows="4" class="w-full px-3 py-2 bg-neutral-950 border border-neutral-800 rounded text-sm text-white focus:outline-none focus:border-sky-500"></textarea>
        </div>

        <div>
          <label for="inv-evidence" class="block text-xs font-medium text-neutral-400 uppercase mb-1">Evidence Payload (JSON)</label>
          <textarea id="inv-evidence" bind:value={newInvestigation.evidence} placeholder="[]" rows="2" class="w-full px-3 py-2 bg-neutral-950 border border-neutral-800 rounded text-sm text-neutral-400 font-mono focus:outline-none focus:border-sky-500"></textarea>
        </div>

        <div class="flex justify-end gap-2 pt-2 border-t border-neutral-800 mt-4">
          <button 
            type="button"
            onclick={() => isAddModalOpen = false} 
            class="px-4 py-2 bg-neutral-800 hover:bg-neutral-700 text-neutral-300 rounded text-sm font-medium mt-2">
            Cancel
          </button>
          <button 
            type="submit"
            class="px-4 py-2 bg-sky-600 hover:bg-sky-500 text-white rounded text-sm font-medium mt-2">
            Save
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}
