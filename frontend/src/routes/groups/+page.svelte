<script lang="ts">
  import { onMount } from 'svelte';
  import { listGroups, saveGroup, deleteGroup, type GroupRecord } from '$lib/api/groups';
  import { listHosts, type HostRecord } from '$lib/api/hosts';

  let isAddModalOpen = $state(false);
  let backendAvailable = $state(true);
  let groups = $state<GroupRecord[]>([]);
  let availableHosts = $state<HostRecord[]>([]);

  let newGroup = $state({
    name: "",
    color: "#3b82f6",
    selectedHosts: [] as string[]
  });

  const colors = ["#ef4444", "#f97316", "#f59e0b", "#10b981", "#3b82f6", "#8b5cf6", "#ec4899"];

  onMount(async () => {
    try {
      [groups, availableHosts] = await Promise.all([listGroups(), listHosts()]);
    } catch {
      backendAvailable = false;
    }
  });

  function hostLabel(id: string) {
    return availableHosts.find((h) => h.id === id)?.label ?? id;
  }

  async function createGroup(e: Event) {
    e.preventDefault();
    if (!newGroup.name) return;

    try {
      const saved = await saveGroup({
        name: newGroup.name,
        color: newGroup.color,
        hostIds: newGroup.selectedHosts
      });
      groups = [...groups, saved];
    } catch {
      backendAvailable = false;
    }

    newGroup = { name: "", color: "#3b82f6", selectedHosts: [] };
    isAddModalOpen = false;
  }

  async function removeGroup(id: string) {
    try {
      await deleteGroup(id);
      groups = groups.filter((g) => g.id !== id);
    } catch {
      backendAvailable = false;
    }
  }

  function toggleHost(hostId: string) {
    if (newGroup.selectedHosts.includes(hostId)) {
      newGroup.selectedHosts = newGroup.selectedHosts.filter(id => id !== hostId);
    } else {
      newGroup.selectedHosts.push(hostId);
    }
  }
</script>

<div class="max-w-5xl mx-auto space-y-6">
  <div class="flex justify-between items-center">
    <div>
      <h1 class="text-2xl font-bold text-white tracking-tight">Host Groups</h1>
      <p class="text-neutral-400 text-sm mt-1">Organize servers into logical clusters for batch actions and multi-pane management.</p>
      {#if !backendAvailable}
        <p class="text-amber-500 text-xs mt-1">Tauri backend not detected — changes won't be saved to disk.</p>
      {/if}
    </div>
    <button 
      onclick={() => isAddModalOpen = true}
      class="px-4 py-2 bg-sky-600 hover:bg-sky-500 text-white font-medium text-sm rounded-md transition-colors flex items-center gap-2">
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"></path></svg>
      Create Group
    </button>
  </div>

  <!-- Groups Grid -->
  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
    {#each groups as group}
      <div class="bg-neutral-900 border border-neutral-800 rounded-lg p-5 flex flex-col justify-between hover:border-neutral-700 transition-colors">
        <div>
          <div class="flex items-center gap-3">
            <span class="w-4 h-4 rounded-full inline-block" style="background-color: {group.color}"></span>
            <h3 class="font-semibold text-white text-lg">{group.name}</h3>
          </div>
          <p class="text-neutral-400 text-sm mt-2">{group.hostIds.length} hosts assigned</p>
          {#if group.hostIds.length > 0}
            <div class="flex gap-1 flex-wrap mt-2">
              {#each group.hostIds as hostId}
                <span class="px-2 py-0.5 bg-neutral-800 border border-neutral-700 text-neutral-400 rounded text-xs">{hostLabel(hostId)}</span>
              {/each}
            </div>
          {/if}
        </div>
        <div class="mt-4 pt-3 border-t border-neutral-800/80 flex justify-end gap-2">
          <button onclick={() => removeGroup(group.id)} class="px-3 py-1 bg-neutral-800 hover:bg-red-600 hover:text-white rounded text-xs text-neutral-300 transition-colors">Delete</button>
          <button class="px-3 py-1 bg-sky-600/20 text-sky-400 hover:bg-sky-600 hover:text-white rounded text-xs transition-colors">Launch All</button>
        </div>
      </div>
    {/each}
  </div>
</div>

<!-- Create Group Modal -->
{#if isAddModalOpen}
  <div class="fixed inset-0 bg-black/70 flex items-center justify-center p-4 z-50">
    <div class="bg-neutral-900 border border-neutral-800 rounded-lg p-6 max-w-lg w-full space-y-4">
      <h3 class="text-xl font-bold text-white">Create Host Group</h3>
      
      <form onsubmit={createGroup} class="space-y-4">
        <div>
          <label for="group-name" class="block text-xs font-medium text-neutral-400 uppercase mb-1">Group Name</label>
          <input id="group-name" bind:value={newGroup.name} required placeholder="e.g. Database Cluster" class="w-full px-3 py-2 bg-neutral-950 border border-neutral-800 rounded text-sm text-white focus:outline-none focus:border-sky-500" />
        </div>

        <div>
          <span class="block text-xs font-medium text-neutral-400 uppercase mb-2">Color Swatch</span>
          <div class="flex gap-2">
            {#each colors as c}
              <button 
                type="button"
                onclick={() => newGroup.color = c}
                class="w-7 h-7 rounded-full transition-transform border-2 {newGroup.color === c ? 'border-white scale-110' : 'border-transparent'}" 
                style="background-color: {c}"
                aria-label="Select color {c}">
              </button>
            {/each}
          </div>
        </div>

        <div>
          <span class="block text-xs font-medium text-neutral-400 uppercase mb-2">Assign Hosts</span>
          <div class="space-y-1 max-h-40 overflow-y-auto p-2 bg-neutral-950 border border-neutral-800 rounded">
            {#each availableHosts as h}
              <label class="flex items-center gap-2 p-1.5 hover:bg-neutral-900 rounded cursor-pointer text-sm text-neutral-300">
                <input 
                  type="checkbox" 
                  checked={newGroup.selectedHosts.includes(h.id)} 
                  onchange={() => toggleHost(h.id)}
                  class="rounded bg-neutral-900 border-neutral-700 text-sky-600 focus:ring-sky-500"
                />
                <span>{h.label}</span>
              </label>
            {/each}
          </div>
        </div>

        <div class="flex justify-end gap-2 pt-2">
          <button 
            type="button"
            onclick={() => isAddModalOpen = false} 
            class="px-4 py-2 bg-neutral-800 hover:bg-neutral-700 text-neutral-300 rounded text-sm font-medium">
            Cancel
          </button>
          <button 
            type="submit"
            class="px-4 py-2 bg-sky-600 hover:bg-sky-500 text-white rounded text-sm font-medium">
            Save Group
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}