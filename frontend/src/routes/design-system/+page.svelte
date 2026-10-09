<script lang="ts">
  import { onMount } from 'svelte';
  import { Button, Badge, Input, Card } from '$lib/components/ui';
  import PageHeader from '$lib/components/PageHeader.svelte';

  // Live Brand Accent Switcher for Pro Theme Testing
  const brandAccents = [
    { name: 'Monochrome (Zinc)', hex: '#71717a' },
    { name: 'Cyan (CATerm / CAMark)', hex: '#06b6d4' },
    { name: 'Emerald (CACash)', hex: '#10b981' },
    { name: 'Violet (CAStudio)', hex: '#8b5cf6' },
    { name: 'Rose (Alert)', hex: '#f43f5e' },
    { name: 'Blue (Core)', hex: '#3b82f6' },
  ];

  let selectedAccent = $state(brandAccents[1].hex);
  let buttonLoading = $state(false);
  let testInputValue = $state('');

  function setBrandAccent(hex: string) {
    selectedAccent = hex;
    if (typeof document !== 'undefined') {
      document.documentElement.style.setProperty('--ca-brand', hex);
    }
  }

  onMount(() => {
    setBrandAccent(selectedAccent);
  });
</script>

<div class="flex-1 flex flex-col h-full bg-[#0A0A0C] text-[#EDEDED] overflow-y-auto p-6 space-y-8 font-sans select-none">
  <!-- Top Navigation Header -->
  <PageHeader
    title="CADS v1.0 Design System Showcase"
    description="Interactive living component library and design token contract for CATerm and CA Group applications."
  />

  <!-- Pro Brand Accent Live Switcher -->
  <Card
    title="Pro Custom Theming (CSS Variable --ca-brand)"
    description="Test how the monochrome base transitions seamlessly when an active Pro accent color is applied."
  >
    <div class="flex flex-wrap items-center gap-2 pt-1">
      {#each brandAccents as b}
        <button
          onclick={() => setBrandAccent(b.hex)}
          class="flex items-center gap-2 px-3 py-1.5 rounded-lg border text-xs font-medium transition-all cursor-pointer {selectedAccent === b.hex ? 'border-white bg-white/10 text-white shadow-xs' : 'border-[#272732] bg-[#18181F]/40 text-neutral-400 hover:text-white'}"
        >
          <span class="w-3 h-3 rounded-full border border-black/30" style="background-color: {b.hex};"></span>
          <span>{b.name}</span>
          {#if selectedAccent === b.hex}
            <span class="text-[10px] text-white">✓</span>
          {/if}
        </button>
      {/each}
    </div>
  </Card>

  <!-- 1. Buttons Laboratory -->
  <Card
    title="1. Button Primitives (Full Outline & Monochrome Scale)"
    description="Standardized interactive buttons. Strict height and padding scale (sm: 28px, md: 36px, lg: 42px). Solid white is reserved for primary confirm, all other actions use monoline outline."
  >
    <div class="space-y-6">
      <!-- Variants Row -->
      <div>
        <span class="text-[10px] font-mono uppercase text-neutral-500 block mb-2 font-bold tracking-wider">
          Style Variants (size="md"):
        </span>
        <div class="flex flex-wrap items-center gap-3">
          <Button variant="primary">Primary (Solid White)</Button>
          <Button variant="secondary">Secondary (Subtle)</Button>
          <Button variant="outline">Outline (Wireframe)</Button>
          <Button variant="ghost">Ghost (Flat)</Button>
          <Button variant="danger">Danger (Rose)</Button>
          <Button variant="brand">Brand (--ca-brand)</Button>
        </div>
      </div>

      <!-- Size Scale Row -->
      <div>
        <span class="text-[10px] font-mono uppercase text-neutral-500 block mb-2 font-bold tracking-wider">
          Size Scale (variant="outline" & "brand"):
        </span>
        <div class="flex flex-wrap items-center gap-3">
          <Button variant="outline" size="sm">Small (sm · 28px)</Button>
          <Button variant="outline" size="md">Medium (md · 36px)</Button>
          <Button variant="outline" size="lg">Large (lg · 42px)</Button>
          <Button variant="brand" size="icon" title="Terminal Icon">
            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 9l3 3-3 3m5 0h3" />
            </svg>
          </Button>
        </div>
      </div>

      <!-- State Tester Row -->
      <div>
        <span class="text-[10px] font-mono uppercase text-neutral-500 block mb-2 font-bold tracking-wider">
          Interactive States (Disabled & Loading):
        </span>
        <div class="flex flex-wrap items-center gap-3">
          <Button variant="primary" disabled>Disabled Primary</Button>
          <Button variant="outline" disabled>Disabled Outline</Button>
          <Button
            variant="brand"
            loading={buttonLoading}
            onclick={() => {
              buttonLoading = true;
              setTimeout(() => (buttonLoading = false), 2000);
            }}
          >
            {buttonLoading ? 'Processing...' : 'Click to Test Loading Spinner'}
          </Button>
        </div>
      </div>
    </div>
  </Card>

  <!-- 2. Badges & Tags Laboratory -->
  <Card
    title="2. Badge & Status Primitives"
    description="Micro badges for host states, connection metrics, protocol tags, and audit flags."
  >
    <div class="space-y-4">
      <div>
        <span class="text-[10px] font-mono uppercase text-neutral-500 block mb-2 font-bold tracking-wider">
          Variants & Colors:
        </span>
        <div class="flex flex-wrap items-center gap-2">
          <Badge variant="neutral">NEUTRAL 200</Badge>
          <Badge variant="success">● ONLINE / CONNECTED</Badge>
          <Badge variant="warning">▲ HIGH LOAD 88%</Badge>
          <Badge variant="danger">✕ CRITICAL DISCONNECT</Badge>
          <Badge variant="brand">★ PRO ACTIVE</Badge>
        </div>
      </div>

      <div>
        <span class="text-[10px] font-mono uppercase text-neutral-500 block mb-2 font-bold tracking-wider">
          Sizes (xs vs sm):
        </span>
        <div class="flex items-center gap-2">
          <Badge variant="neutral" size="xs">SIZE XS (10px)</Badge>
          <Badge variant="neutral" size="sm">SIZE SM (12px)</Badge>
          <Badge variant="brand" size="xs">ED25519-ZK</Badge>
          <Badge variant="brand" size="sm">HOST-PRO-TIER</Badge>
        </div>
      </div>
    </div>
  </Card>

  <!-- 3. Form Input Controls -->
  <Card
    title="3. Input & Text Controls"
    description="Monochrome text input with subtle border transition, icon slots, and trailing action buttons."
  >
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <div>
        <label for="demo-input-1" class="block text-xs font-semibold text-neutral-300 mb-1.5">Standard Input:</label>
        <Input id="demo-input-1" bind:value={testInputValue} placeholder="e.g. 192.168.1.100 or server.domain.com" />
      </div>

      <div>
        <label for="demo-input-2" class="block text-xs font-semibold text-neutral-300 mb-1.5">Input with Search Icon:</label>
        <Input id="demo-input-2" placeholder="Search hosts, snippets, logs...">
          {#snippet leadingIcon()}
            <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
          {/snippet}
        </Input>
      </div>
    </div>
  </Card>

  <!-- 4. Design Token Specifications & Contracts -->
  <Card
    title="4. CADS v1.0 Architectural Design Contract"
    description="Core tokens adhering to Sovereign Developer ethos: Zero-knowledge, Anti-bloat, Local-first."
  >
    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 text-xs font-mono">
      <div class="p-3 rounded-lg bg-[#0E0E12] border border-[#272732] space-y-1">
        <span class="text-neutral-500 block text-[10px]">SURFACE_BASE</span>
        <span class="font-bold text-white block">#0A0A0C</span>
        <span class="text-[10px] text-neutral-400">Deep obsidian background</span>
      </div>
      <div class="p-3 rounded-lg bg-[#0E0E12] border border-[#272732] space-y-1">
        <span class="text-neutral-500 block text-[10px]">SURFACE_ELEVATED</span>
        <span class="font-bold text-white block">#121217 / #161616</span>
        <span class="text-[10px] text-neutral-400">Raised content cards</span>
      </div>
      <div class="p-3 rounded-lg bg-[#0E0E12] border border-[#272732] space-y-1">
        <span class="text-neutral-500 block text-[10px]">BORDER_SUBTLE</span>
        <span class="font-bold text-white block">#272732 / #27272A</span>
        <span class="text-[10px] text-neutral-400">Crisp HUD wireframe lines</span>
      </div>
      <div class="p-3 rounded-lg bg-[#0E0E12] border border-[#272732] space-y-1">
        <span class="text-neutral-500 block text-[10px]">BRAND_ACCENT</span>
        <span class="font-bold block" style="color: {selectedAccent};">{selectedAccent.toUpperCase()}</span>
        <span class="text-[10px] text-neutral-400">Reactive CSS variable</span>
      </div>
    </div>
  </Card>
</div>
