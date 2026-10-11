<script lang="ts">
  import { onMount } from 'svelte';
  import { 
    Button, 
    Badge, 
    Input, 
    Card, 
    Table, 
    Modal, 
    Alert, 
    Sidebar, 
    SidebarItem, 
    Icon, 
    LanguageSwitcher, 
    ThemeSwitcher 
  } from '$lib/components/ui';
  import { 
    SplashScreen, 
    LoginScreen, 
    DashboardView, 
    AiChatPanel 
  } from '$lib/components/caui/templates';
  import PageHeader from '$lib/components/PageHeader.svelte';

  // Navigation Tabs for Showcase
  type TabType = 'primitives' | 'data-overlays' | 'templates';
  let activeTab = $state<TabType>('primitives');

  // Brand Accents for Pro Theme Testing
  const brandAccents = [
    { name: 'Monochrome (Zinc)', hex: '#71717a' },
    { name: 'Rose / Crimson (CATerm)', hex: '#ef4444' },
    { name: 'Cyan (CAMark)', hex: '#06b6d4' },
    { name: 'Emerald (CACash)', hex: '#10b981' },
    { name: 'Violet (CAStudio)', hex: '#8b5cf6' },
    { name: 'Amber (CAEntech)', hex: '#eab308' },
    { name: 'Blue (Core)', hex: '#3b82f6' },
  ];

  let selectedAccent = $state(brandAccents[1].hex);
  let buttonLoading = $state(false);
  let testInputValue = $state('');
  let modalOpen = $state(false);
  let activeTemplate = $state<'splash' | 'login' | 'dashboard' | 'aichat'>('dashboard');

  // Sample Data for Table
  const tableColumns = [
    { key: 'host', label: 'Host & Address', sortable: true },
    { key: 'status', label: 'Status' },
    { key: 'cpu', label: 'CPU Load', sortable: true },
    { key: 'memory', label: 'RAM Usage', sortable: true },
    { key: 'uptime', label: 'Uptime' },
  ];

  const tableData = [
    { host: 'Hostinger VPS (100.76.150.46)', status: 'online', cpu: '12%', memory: '3.4 / 8.0 GB', uptime: '42d 18h' },
    { host: 'X1 ThinkPad (100.78.73.124)', status: 'online', cpu: '24%', memory: '7.8 / 16.0 GB', uptime: '6d 04h' },
    { host: 'pc-gerlink (100.64.12.89)', status: 'idle', cpu: '4%', memory: '2.1 / 16.0 GB', uptime: '18d 22h' },
    { host: 'AWS Backup Node (ap-southeast-1)', status: 'offline', cpu: '0%', memory: '0.0 / 4.0 GB', uptime: 'Offline' },
  ];

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

<div class="flex-1 flex flex-col h-full bg-[#0A0A0C] text-[#EDEDED] overflow-y-auto p-6 space-y-6 font-sans select-none">
  <!-- Top Navigation Header -->
  <div class="flex flex-col md:flex-row md:items-center justify-between gap-4 border-b border-[#272732] pb-5">
    <PageHeader
      title="CAUI v1.1 — CAFramework Design System"
      description="Design tokens, primitives, overlays, and full-screen templates for all 17 CA applications."
    />
    
    <!-- Tab Controls -->
    <div class="flex items-center gap-1 bg-[#141419] p-1 rounded-lg border border-[#272732]">
      <button
        onclick={() => (activeTab = 'primitives')}
        class="px-3 py-1.5 rounded-md text-xs font-medium transition-all {activeTab === 'primitives' ? 'bg-[#272732] text-white shadow-xs' : 'text-neutral-400 hover:text-white'}"
      >
        Atomic Primitives
      </button>
      <button
        onclick={() => (activeTab = 'data-overlays')}
        class="px-3 py-1.5 rounded-md text-xs font-medium transition-all {activeTab === 'data-overlays' ? 'bg-[#272732] text-white shadow-xs' : 'text-neutral-400 hover:text-white'}"
      >
        Data & Overlays
      </button>
      <button
        onclick={() => (activeTab = 'templates')}
        class="px-3 py-1.5 rounded-md text-xs font-medium transition-all {activeTab === 'templates' ? 'bg-[#272732] text-white shadow-xs' : 'text-neutral-400 hover:text-white'}"
      >
        Screen Templates
      </button>
    </div>
  </div>

  <!-- Pro Brand Accent Live Switcher Bar -->
  <Card title="Brand Accent Theming (CSS Variable --ca-brand)" description="Live override palette for Pro custom branding.">
    <div class="flex flex-wrap items-center justify-between gap-3 pt-1">
      <div class="flex flex-wrap items-center gap-2">
        {#each brandAccents as b}
          <button
            onclick={() => setBrandAccent(b.hex)}
            class="flex items-center gap-2 px-2.5 py-1.5 rounded-lg border text-xs font-medium transition-all cursor-pointer {selectedAccent === b.hex ? 'border-white bg-white/10 text-white' : 'border-[#272732] bg-[#18181F]/40 text-neutral-400 hover:text-white'}"
          >
            <span class="w-3 h-3 rounded-full border border-black/30" style="background-color: {b.hex};"></span>
            <span>{b.name}</span>
            {#if selectedAccent === b.hex}<span class="text-[10px] text-white">✓</span>{/if}
          </button>
        {/each}
      </div>
      
      <div class="flex items-center gap-3">
        <LanguageSwitcher />
        <ThemeSwitcher />
      </div>
    </div>
  </Card>

  <!-- TAB 1: ATOMIC PRIMITIVES -->
  {#if activeTab === 'primitives'}
    <!-- Buttons -->
    <Card title="1. Button Scale & Variants" description="Monochrome solid base with outline secondary and dynamic brand variant.">
      <div class="space-y-6">
        <div>
          <span class="text-[10px] font-mono uppercase text-neutral-500 block mb-2 font-bold tracking-wider">Style Variants (size="md"):</span>
          <div class="flex flex-wrap items-center gap-3">
            <Button variant="primary">Primary (Solid White)</Button>
            <Button variant="secondary">Secondary (Dark)</Button>
            <Button variant="outline">Outline (Wireframe)</Button>
            <Button variant="ghost">Ghost</Button>
            <Button variant="danger">Danger</Button>
            <Button variant="brand">Brand Accent</Button>
          </div>
        </div>

        <div>
          <span class="text-[10px] font-mono uppercase text-neutral-500 block mb-2 font-bold tracking-wider">Size Scale:</span>
          <div class="flex flex-wrap items-center gap-3">
            <Button variant="outline" size="sm">Small (28px)</Button>
            <Button variant="outline" size="md">Medium (36px)</Button>
            <Button variant="outline" size="lg">Large (42px)</Button>
            <Button
              variant="brand"
              loading={buttonLoading}
              onclick={() => {
                buttonLoading = true;
                setTimeout(() => (buttonLoading = false), 2000);
              }}
            >
              {buttonLoading ? 'Executing...' : 'Click for Loading State'}
            </Button>
          </div>
        </div>
      </div>
    </Card>

    <!-- Badges & Inputs -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
      <Card title="2. Badges & Status Indicators" description="Status tags for hosts and telemetry.">
        <div class="space-y-3">
          <div class="flex flex-wrap items-center gap-2">
            <Badge variant="neutral">NEUTRAL 200</Badge>
            <Badge variant="success">● ONLINE</Badge>
            <Badge variant="warning">▲ HIGH LOAD</Badge>
            <Badge variant="danger">✕ CRITICAL</Badge>
            <Badge variant="brand">★ PRO ACTIVE</Badge>
          </div>
          <div class="flex items-center gap-2 pt-2">
            <Badge variant="brand" size="xs">ED25519-ZK</Badge>
            <Badge variant="neutral" size="sm">ARM64 ARCH</Badge>
            <Badge variant="success" size="sm">TAILSCALE MESH</Badge>
          </div>
        </div>
      </Card>

      <Card title="3. Form Inputs" description="Monochrome input controls with search icon slots.">
        <div class="space-y-3">
          <Input bind:value={testInputValue} placeholder="e.g. 100.76.150.46 or user@host" />
          <Input placeholder="Search logs, sessions, commands...">
            {#snippet leadingIcon()}
              <Icon name="search" size={14} class="text-neutral-400" />
            {/snippet}
          </Input>
        </div>
      </Card>
    </div>

  <!-- TAB 2: DATA & OVERLAYS -->
  {:else if activeTab === 'data-overlays'}
    <!-- Alerts -->
    <Card title="1. Alert & Banner Notifications" description="Inline notifications for system events and warnings.">
      <div class="space-y-3">
        <Alert variant="info" title="Zero-Knowledge Enclave Active">
          All session keys and SQLite telemetry are encrypted locally via Argon2id.
        </Alert>
        <Alert variant="warning" title="SSH Host Connection High Latency">
          Remote host response time exceeded 280ms over current relay route.
        </Alert>
        <Alert variant="danger" title="Unauthorized Sudo Attempt Detected">
          Audit rule violation triggered in container <code>docker-prod-db</code>.
        </Alert>
      </div>
    </Card>

    <!-- Table Data Grid -->
    <Card title="2. Data Grid & Telemetry Table" description="Sortable table with row hover, badges, and action slots.">
      <Table columns={tableColumns} data={tableData}>
        {#snippet row(item: any)}
          <tr class="border-b border-[#272732] hover:bg-white/[0.02] transition-colors text-xs font-mono">
            <td class="px-4 py-3 text-white font-medium">{item.host}</td>
            <td class="px-4 py-3">
              {#if item.status === 'online'}
                <Badge variant="success">ONLINE</Badge>
              {:else if item.status === 'idle'}
                <Badge variant="neutral">IDLE</Badge>
              {:else}
                <Badge variant="danger">OFFLINE</Badge>
              {/if}
            </td>
            <td class="px-4 py-3 text-neutral-300">{item.cpu}</td>
            <td class="px-4 py-3 text-neutral-300">{item.memory}</td>
            <td class="px-4 py-3 text-neutral-400">{item.uptime}</td>
          </tr>
        {/snippet}
      </Table>
    </Card>

    <!-- Modal Trigger -->
    <Card title="3. Dialog / Modal / Popup" description="Backdrop blur modal with frameless header.">
      <div class="flex items-center justify-between">
        <span class="text-xs text-neutral-400">Click to preview interactive dialog overlay:</span>
        <Button variant="outline" onclick={() => (modalOpen = true)}>Open Test Modal</Button>
      </div>

      <Modal bind:open={modalOpen} title="CAFramework Security Confirmation" size="md">
        <div class="space-y-4 text-xs text-neutral-300">
          <p>Are you sure you want to deploy cryptographic policy across all 17 local instances?</p>
          <Alert variant="info">This operation will synchronize tokens without transmitting private keys.</Alert>
        </div>
        {#snippet footer()}
          <div class="flex items-center justify-end gap-2">
            <Button variant="ghost" size="sm" onclick={() => (modalOpen = false)}>Cancel</Button>
            <Button variant="primary" size="sm" onclick={() => (modalOpen = false)}>Confirm & Apply</Button>
          </div>
        {/snippet}
      </Modal>
    </Card>

  <!-- TAB 3: SCREEN TEMPLATES -->
  {:else if activeTab === 'templates'}
    <!-- Template Selector -->
    <div class="flex items-center justify-between bg-[#111116] p-3 rounded-xl border border-[#272732]">
      <span class="text-xs font-medium text-neutral-300">Select Template to Preview:</span>
      <div class="flex items-center gap-2">
        <button
          onclick={() => (activeTemplate = 'dashboard')}
          class="px-3 py-1 rounded-md text-xs font-medium transition-all {activeTemplate === 'dashboard' ? 'bg-[var(--ca-brand,#ef4444)] text-white' : 'text-neutral-400 hover:text-white bg-[#1A1A22]'}"
        >
          Dashboard View
        </button>
        <button
          onclick={() => (activeTemplate = 'login')}
          class="px-3 py-1 rounded-md text-xs font-medium transition-all {activeTemplate === 'login' ? 'bg-[var(--ca-brand,#ef4444)] text-white' : 'text-neutral-400 hover:text-white bg-[#1A1A22]'}"
        >
          CAFramework Login
        </button>
        <button
          onclick={() => (activeTemplate = 'splash')}
          class="px-3 py-1 rounded-md text-xs font-medium transition-all {activeTemplate === 'splash' ? 'bg-[var(--ca-brand,#ef4444)] text-white' : 'text-neutral-400 hover:text-white bg-[#1A1A22]'}"
        >
          Splash Screen
        </button>
        <button
          onclick={() => (activeTemplate = 'aichat')}
          class="px-3 py-1 rounded-md text-xs font-medium transition-all {activeTemplate === 'aichat' ? 'bg-[var(--ca-brand,#ef4444)] text-white' : 'text-neutral-400 hover:text-white bg-[#1A1A22]'}"
        >
          AI Co-Pilot Chat
        </button>
      </div>
    </div>

    <!-- Active Template View Frame -->
    <div class="border border-[#272732] rounded-xl overflow-hidden bg-[#0A0A0C] min-h-[520px] relative shadow-2xl">
      {#if activeTemplate === 'dashboard'}
        <DashboardView />
      {:else if activeTemplate === 'login'}
        <LoginScreen />
      {:else if activeTemplate === 'splash'}
        <SplashScreen />
      {:else if activeTemplate === 'aichat'}
        <AiChatPanel />
      {/if}
    </div>
  {/if}
</div>
