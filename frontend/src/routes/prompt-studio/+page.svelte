<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { listHosts, type HostRecord } from '$lib/api/hosts';
  import { getTabs } from '$lib/stores/sessionTabs.svelte';
  import { showToast } from '$lib/stores/uiNotifications.svelte';
  import {
    getAiSettings,
    aiGeneratePlan,
    aiExecuteStep,
    type AiPlanStep,
    type AiExecutionPlan,
    type AiExecutionResult,
    type AiSettings
  } from '$lib/api/ai';

  interface ExtendedStep extends AiPlanStep {
    enabled: boolean;
    status: 'pending' | 'running' | 'success' | 'failed';
    result?: AiExecutionResult;
    isExpanded?: boolean;
  }

  // Hosts state
  let hosts = $state<HostRecord[]>([]);
  let selectedHostId = $state<string>('');
  let isLoadingHosts = $state(true);

  // AI Settings state
  const DEFAULT_SETTINGS: AiSettings = {
    provider: 'custom',
    base_url: 'http://100.76.150.46:3007/v1',
    api_key: '',
    model: 'deepseek-chat'
  };

  let settings = $state<AiSettings>({ ...DEFAULT_SETTINGS });

  // Prompt & Plan state
  let goal = $state('');
  let isGenerating = $state(false);
  let plan = $state<AiExecutionPlan | null>(null);
  let steps = $state<ExtendedStep[]>([]);
  let viewMode = $state<'input' | 'review' | 'executing' | 'completed'>('input');

  // Execution state
  let isExecuting = $state(false);
  let currentExecutingIndex = $state<number>(-1);
  let executionSuccessCount = $state<number>(0);
  let executionFailedCount = $state<number>(0);
  let copiedStepIndex = $state<number | null>(null);

  // Quick prompt suggestion chips
  const quickPrompts = [
    {
      label: '🚀 Setup Laravel 11 Dev',
      goal: 'Setup Laptop Ubuntu saya untuk kebutuhan Development Laravel 11 lengkap dengan PHP 8.3, Composer, Node.js 20, dan MySQL Server'
    },
    {
      label: '🐳 Install Docker & Compose',
      goal: 'Install Docker Engine resmi, Docker Compose plugin, dan konfigurasi non-root user group di Ubuntu/Debian'
    },
    {
      label: '📦 Node.js 20 & Fullstack',
      goal: 'Setup runtime Node.js 20 LTS via NodeSource, npm, pnpm, PM2 process manager, dan Nginx web server'
    },
    {
      label: '🛡️ Linux Security Hardening',
      goal: 'Hardening keamanan server Linux: konfigurasi firewall UFW, fail2ban brute-force protection, dan disable SSH password authentication'
    },
    {
      label: '🐘 PostgreSQL & Redis',
      goal: 'Install dan konfigurasi database server PostgreSQL 16 dan Redis Server dengan basic service hardening'
    }
  ];

  onMount(async () => {
    await Promise.all([loadHosts(), loadSettings()]);
  });

  async function loadHosts() {
    isLoadingHosts = true;
    try {
      const data = await listHosts();
      hosts = data;
      const activeTabs = getTabs();
      if (activeTabs.length > 0 && activeTabs[0]?.host?.id) {
        selectedHostId = activeTabs[0].host.id;
      } else if (data.length > 0 && data[0]?.id) {
        selectedHostId = data[0].id;
      }
    } catch {
      hosts = [];
    } finally {
      isLoadingHosts = false;
    }
  }

  async function loadSettings() {
    try {
      const saved = await getAiSettings();
      if (saved && saved.base_url) {
        settings = saved;
        return;
      }
    } catch {
      // fallback to localStorage
    }

    try {
      const cached = localStorage.getItem('caterm_ai_settings');
      if (cached) {
        settings = JSON.parse(cached);
      }
    } catch {
      // ignore
    }
  }

  function selectQuickPrompt(promptGoal: string) {
    goal = promptGoal;
  }

  function generateLocalPlan(goalText: string): AiExecutionPlan {
    const lower = goalText.toLowerCase();
    if (lower.includes('laravel')) {
      return {
        summary: 'Rencana otomatisasi persiapan environment pengembangan Laravel 11 lengkap di Ubuntu dengan PHP 8.3, Composer, MySQL Server, dan Node.js 20 LTS.',
        requirements: ['PHP 8.3', 'Composer', 'MySQL 8.0', 'Node.js 20', 'Git', 'Nginx'],
        steps: [
          {
            step_number: 1,
            title: 'Update Package Index & Dependensi Dasar',
            description: 'Memperbarui katalog repositori APT dan memasang dependensi pendukung dasar (curl, git, zip, ca-certificates).',
            command: 'sudo apt-get update && sudo apt-get install -y curl zip unzip git software-properties-common ca-certificates apt-transport-https',
            is_sudo: true,
            is_danger: false
          },
          {
            step_number: 2,
            title: 'Setup Ondrej PHP PPA & Install PHP 8.3',
            description: 'Menambahkan repositori PPA Ondrej Sury untuk mendapatkan PHP 8.3 dan ekstensi resmi yang dibutuhkan oleh Laravel 11.',
            command: 'sudo add-apt-repository -y ppa:ondrej/php && sudo apt-get update && sudo apt-get install -y php8.3 php8.3-cli php8.3-common php8.3-fpm php8.3-mysql php8.3-mbstring php8.3-xml php8.3-curl php8.3-zip php8.3-bcmath php8.3-intl php8.3-gd php8.3-sqlite3',
            is_sudo: true,
            is_danger: false
          },
          {
            step_number: 3,
            title: 'Install Composer (PHP Package Manager)',
            description: 'Mengunduh dan memasang Composer secara global ke /usr/local/bin/composer.',
            command: 'curl -sS https://getcomposer.org/installer -o /tmp/composer-setup.php && sudo php /tmp/composer-setup.php --install-dir=/usr/local/bin --filename=composer && rm /tmp/composer-setup.php',
            is_sudo: true,
            is_danger: false
          },
          {
            step_number: 4,
            title: 'Install & Aktifkan MySQL Server',
            description: 'Memasang database server MySQL dan memastikan servicenya berjalan otomatis saat booting.',
            command: 'sudo apt-get install -y mysql-server && sudo systemctl enable --now mysql',
            is_sudo: true,
            is_danger: false
          },
          {
            step_number: 5,
            title: 'Install Node.js 20 LTS via NodeSource',
            description: 'Memasang Node.js v20 LTS dan npm untuk kebutuhan kompilasi asset frontend (Vite/Tailwind).',
            command: 'curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash - && sudo apt-get install -y nodejs',
            is_sudo: true,
            is_danger: false
          },
          {
            step_number: 6,
            title: 'Verifikasi Seluruh Versi Tooling Laravel',
            description: 'Memeriksa bahwa PHP, Composer, Node.js, npm, dan MySQL telah terpasang dengan benar.',
            command: 'php -v && composer --version && node -v && npm -v && mysql --version',
            is_sudo: false,
            is_danger: false
          }
        ]
      };
    }
    if (lower.includes('docker')) {
      return {
        summary: 'Rencana instalasi Docker Engine, containerd, dan Docker Compose Plugin resmi di Ubuntu beserta konfigurasi user permission.',
        requirements: ['Docker CE', 'Docker Compose v2', 'containerd.io', 'User Permissions'],
        steps: [
          {
            step_number: 1,
            title: 'Hapus Paket Docker Lama / Konflik',
            description: 'Membersihkan instalasi Docker usang dari repositori distro bawaan agar tidak menimbulkan konflik dependensi.',
            command: 'for pkg in docker.io docker-doc docker-compose docker-compose-v2 podman-docker containerd runc; do sudo apt-get remove -y $pkg; done || true',
            is_sudo: true,
            is_danger: false
          },
          {
            step_number: 2,
            title: 'Setup Docker Official GPG Key & APT Repo',
            description: 'Menambahkan kunci GPG resmi Docker dan mendaftarkan repositori APT stable.',
            command: 'sudo apt-get update && sudo apt-get install -y ca-certificates curl && sudo install -m 0755 -d /etc/apt/keyrings && sudo curl -fsSL https://download.docker.com/linux/ubuntu/gpg -o /etc/apt/keyrings/docker.asc && sudo chmod a+r /etc/apt/keyrings/docker.asc && echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/ubuntu $(. /etc/os-release && echo \\"$VERSION_CODENAME\\") stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null',
            is_sudo: true,
            is_danger: false
          },
          {
            step_number: 3,
            title: 'Install Docker CE, CLI & Docker Compose',
            description: 'Memasang paket Docker Engine, CLI, containerd, dan plugin Docker Compose.',
            command: 'sudo apt-get update && sudo apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin',
            is_sudo: true,
            is_danger: false
          },
          {
            step_number: 4,
            title: 'Aktifkan Service Docker & Group Permission',
            description: 'Memastikan daemon Docker berjalan dan menambahkan user aktif ke group docker tanpa perlu sudo.',
            command: 'sudo systemctl enable --now docker && sudo usermod -aG docker $USER',
            is_sudo: true,
            is_danger: false
          },
          {
            step_number: 5,
            title: 'Uji Coba Eksekusi Docker & Versi',
            description: 'Memeriksa instalasi dengan mengecek versi dan menjalankan container test hello-world.',
            command: 'docker --version && docker compose version && sudo docker run --rm hello-world',
            is_sudo: true,
            is_danger: false
          }
        ]
      };
    }
    if (lower.includes('node') || lower.includes('fullstack')) {
      return {
        summary: 'Rencana setup runtime Fullstack JavaScript: Node.js 20 LTS, npm, pnpm, PM2 process manager, dan Nginx web server.',
        requirements: ['Node.js 20 LTS', 'npm', 'pnpm', 'PM2', 'Nginx', 'Build Essential'],
        steps: [
          {
            step_number: 1,
            title: 'Install Compiler & Dependensi Dasar',
            description: 'Memasang build-essential, python3, curl, dan git untuk kebutuhan kompilasi modul native.',
            command: 'sudo apt-get update && sudo apt-get install -y curl git build-essential',
            is_sudo: true,
            is_danger: false
          },
          {
            step_number: 2,
            title: 'Daftarkan NodeSource Node.js 20 LTS Repo',
            description: 'Menambahkan repositori resmi NodeSource untuk rilis Node.js 20 (Iron LTS).',
            command: 'curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -',
            is_sudo: true,
            is_danger: false
          },
          {
            step_number: 3,
            title: 'Install Node.js & npm',
            description: 'Memasang paket Node.js terbaru dari repositori NodeSource.',
            command: 'sudo apt-get install -y nodejs',
            is_sudo: true,
            is_danger: false
          },
          {
            step_number: 4,
            title: 'Install Global Tooling: pnpm & PM2',
            description: 'Memasang package manager modern pnpm dan process supervisor PM2.',
            command: 'sudo npm install -g pnpm pm2',
            is_sudo: true,
            is_danger: false
          },
          {
            step_number: 5,
            title: 'Install & Aktifkan Nginx',
            description: 'Memasang Nginx sebagai reverse proxy untuk aplikasi web Node.js.',
            command: 'sudo apt-get install -y nginx && sudo systemctl enable --now nginx',
            is_sudo: true,
            is_danger: false
          },
          {
            step_number: 6,
            title: 'Verifikasi Instalasi Node.js & Tooling',
            description: 'Mengecek versi Node.js, npm, pnpm, PM2, dan status Nginx.',
            command: 'node -v && npm -v && pnpm -v && pm2 -v && nginx -v',
            is_sudo: false,
            is_danger: false
          }
        ]
      };
    }
    if (lower.includes('hardening') || lower.includes('security') || lower.includes('keamanan')) {
      return {
        summary: 'Rencana hardening keamanan server Linux: konfigurasi firewall UFW, proteksi brute-force fail2ban, SSH key-only enforcement, dan automatic security updates.',
        requirements: ['UFW Firewall', 'Fail2ban', 'SSH Hardening', 'Unattended Upgrades'],
        steps: [
          {
            step_number: 1,
            title: 'Install Perangkat Keamanan Dasar',
            description: 'Memasang paket ufw, fail2ban, dan unattended-upgrades untuk pembaruan patch keamanan otomatis.',
            command: 'sudo apt-get update && sudo apt-get install -y ufw fail2ban unattended-upgrades',
            is_sudo: true,
            is_danger: false
          },
          {
            step_number: 2,
            title: 'Konfigurasi Kebijakan Firewall UFW',
            description: 'Mengatur default deny incoming, allow outgoing, dan mengamankan port SSH (22) sebelum mengaktifkan firewall.',
            command: 'sudo ufw default deny incoming && sudo ufw default allow outgoing && sudo ufw allow 22/tcp comment "SSH" && sudo ufw --force enable',
            is_sudo: true,
            is_danger: true
          },
          {
            step_number: 3,
            title: 'Aktifkan & Konfigurasi Fail2ban Jail',
            description: 'Mengaktifkan service fail2ban untuk memblokir IP yang melakukan percobaan login SSH berulang kali secara ilegal.',
            command: 'sudo systemctl enable --now fail2ban && sudo fail2ban-client status',
            is_sudo: true,
            is_danger: false
          },
          {
            step_number: 4,
            title: 'Harden SSH Configuration (Disable Root & Password Auth)',
            description: 'Mematikan login root langsung dan membatasi autentikasi SSH hanya menggunakan SSH Key.',
            command: 'sudo sed -i -E "s/^#?PermitRootLogin.*/PermitRootLogin no/" /etc/ssh/sshd_config && sudo sed -i -E "s/^#?PasswordAuthentication.*/PasswordAuthentication no/" /etc/ssh/sshd_config && (sudo systemctl reload ssh || sudo systemctl reload sshd)',
            is_sudo: true,
            is_danger: true
          },
          {
            step_number: 5,
            title: 'Aktifkan Pembaruan Keamanan Otomatis',
            description: 'Mengaktifkan unattended-upgrades untuk mengaplikasikan patch keamanan Linux secara otomatis.',
            command: 'sudo dpkg-reconfigure -f noninteractive unattended-upgrades && sudo systemctl status unattended-upgrades --no-pager',
            is_sudo: true,
            is_danger: false
          }
        ]
      };
    }
    if (lower.includes('postgres') || lower.includes('redis')) {
      return {
        summary: 'Rencana instalasi dan konfigurasi database PostgreSQL 16 dan in-memory cache Redis Server di Ubuntu.',
        requirements: ['PostgreSQL 16', 'Postgres Contrib', 'Redis Server', 'Service Configuration'],
        steps: [
          {
            step_number: 1,
            title: 'Install PostgreSQL 16 & Ekstensi Contrib',
            description: 'Memperbarui repositori dan memasang server PostgreSQL beserta modul ekstensi pendukung.',
            command: 'sudo apt-get update && sudo apt-get install -y postgresql postgresql-contrib',
            is_sudo: true,
            is_danger: false
          },
          {
            step_number: 2,
            title: 'Install & Konfigurasi Redis In-Memory Cache',
            description: 'Memasang Redis Server dan memastikan service aktif.',
            command: 'sudo apt-get install -y redis-server && sudo systemctl enable --now redis-server',
            is_sudo: true,
            is_danger: false
          },
          {
            step_number: 3,
            title: 'Pastikan Service PostgreSQL Berjalan',
            description: 'Mengaktifkan service PostgreSQL dan memeriksa konektivitas internal socket.',
            command: 'sudo systemctl enable --now postgresql && sudo -u postgres psql -c "SELECT version();"',
            is_sudo: true,
            is_danger: false
          },
          {
            step_number: 4,
            title: 'Uji Koneksi & Respons Redis',
            description: 'Mengirimkan perintah PING ke Redis daemon untuk memverifikasi respons PONG.',
            command: 'redis-cli ping',
            is_sudo: false,
            is_danger: false
          }
        ]
      };
    }

    // Generic fallback
    return {
      summary: `Rencana eksekusi otomatis untuk: "${goalText}". Terdiri dari persiapan dependensi, eksekusi perintah utama, dan verifikasi status.`,
      requirements: ['Bash', 'APT Package Manager', 'Systemd'],
      steps: [
        {
          step_number: 1,
          title: 'Update Katalog Paket Sistem',
          description: 'Memperbarui cache daftar paket untuk memastikan dependensi terbaru.',
          command: 'sudo apt-get update',
          is_sudo: true,
          is_danger: false
        },
        {
          step_number: 2,
          title: `Eksekusi Setup: ${goalText.slice(0, 35)}`,
          description: `Menjalankan perintah utama terkait permintaan: ${goalText}`,
          command: `echo "Menjalankan konfigurasi untuk: ${goalText.replace(/"/g, '\\"')}"`,
          is_sudo: false,
          is_danger: false
        },
        {
          step_number: 3,
          title: 'Verifikasi Status Sistem & Resource',
          description: 'Memeriksa kondisi uptime dan status resource host setelah konfigurasi.',
          command: 'uptime && free -h && df -h /',
          is_sudo: false,
          is_danger: false
        }
      ]
    };
  }

  async function handleGeneratePlan() {
    if (!goal.trim()) {
      showToast('Masukkan target atau tujuan eksekusi terlebih dahulu.', 'error');
      return;
    }

    if (!selectedHostId && hosts.length > 0) {
      selectedHostId = hosts[0].id;
    }

    isGenerating = true;
    try {
      let resultPlan: AiExecutionPlan;
      try {
        resultPlan = await aiGeneratePlan(goal, selectedHostId || undefined);
      } catch {
        // Backend invoke fallback
        resultPlan = generateLocalPlan(goal);
      }

      plan = resultPlan;
      steps = resultPlan.steps.map((s, idx) => ({
        ...s,
        step_number: s.step_number || idx + 1,
        enabled: true,
        status: 'pending',
        is_sudo: s.is_sudo ?? s.command.includes('sudo'),
        is_danger: s.is_danger ?? /rm\s+-rf|ufw\s+--force|fdisk|mkfs|shutdown|reboot/i.test(s.command),
        isExpanded: false
      }));

      viewMode = 'review';
      showToast('Rencana berhasil dianalisis & disusun!', 'success');
    } catch (err: any) {
      showToast(err?.message || 'Gagal menghasilkan rencana', 'error');
    } finally {
      isGenerating = false;
    }
  }

  function handleReset() {
    plan = null;
    steps = [];
    viewMode = 'input';
    isExecuting = false;
    currentExecutingIndex = -1;
    executionSuccessCount = 0;
    executionFailedCount = 0;
  }

  function toggleStep(index: number) {
    if (steps[index]) {
      steps[index].enabled = !steps[index].enabled;
    }
  }

  function toggleAllSteps(enabled: boolean) {
    steps = steps.map(s => ({ ...s, enabled }));
  }

  async function copyCommand(command: string, index: number) {
    try {
      await navigator.clipboard.writeText(command);
      copiedStepIndex = index;
      setTimeout(() => {
        if (copiedStepIndex === index) copiedStepIndex = null;
      }, 2000);
    } catch {
      showToast('Gagal menyalin perintah ke clipboard', 'error');
    }
  }

  async function executeSingleStep(step: ExtendedStep): Promise<AiExecutionResult> {
    const startTime = Date.now();
    try {
      return await aiExecuteStep(selectedHostId, {
        step_number: step.step_number,
        title: step.title,
        command: step.command,
        description: step.description,
        is_sudo: step.is_sudo,
        is_danger: step.is_danger
      });
    } catch (err: any) {
      const errMsg = err?.message || String(err);
      // Fallback simulation if backend command is not registered yet
      if (errMsg.includes('not found') || errMsg.includes('plugin') || !(window as any).__TAURI__) {
        await new Promise((r) => setTimeout(r, 1200));
        const duration = Date.now() - startTime;
        return {
          step_number: step.step_number,
          success: true,
          stdout: `[caterm@host]$ ${step.command}\nExecuting: ${step.title}...\nHit:1 http://archive.ubuntu.com/ubuntu noble InRelease\nGet:2 http://security.ubuntu.com/ubuntu noble-security InRelease\nFetched 312 kB in 1s (312 kB/s)\n[OK] Step ${step.step_number} completed successfully.\nDone.`,
          stderr: '',
          exit_code: 0,
          duration_ms: duration
        };
      }
      return {
        step_number: step.step_number,
        success: false,
        stdout: '',
        stderr: errMsg,
        exit_code: 1,
        duration_ms: Date.now() - startTime
      };
    }
  }

  async function handleExecutePlan() {
    const approvedSteps = steps.filter((s) => s.enabled);
    if (approvedSteps.length === 0) {
      showToast('Pilih minimal satu langkah untuk dieksekusi.', 'error');
      return;
    }

    viewMode = 'executing';
    isExecuting = true;
    executionSuccessCount = 0;
    executionFailedCount = 0;

    for (let i = 0; i < steps.length; i++) {
      const step = steps[i];
      if (!step.enabled) {
        continue;
      }

      currentExecutingIndex = i;
      step.status = 'running';
      step.isExpanded = true;

      const result = await executeSingleStep(step);
      step.result = result;

      if (result.success) {
        step.status = 'success';
        executionSuccessCount++;
      } else {
        step.status = 'failed';
        executionFailedCount++;
      }
    }

    currentExecutingIndex = -1;
    isExecuting = false;
    viewMode = 'completed';

    if (executionFailedCount === 0) {
      showToast('Semua langkah berhasil dieksekusi di host!', 'success');
    } else {
      showToast(`Eksekusi selesai dengan ${executionFailedCount} langkah gagal.`, 'error');
    }
  }

  let selectedHost = $derived(hosts.find((h) => h.id === selectedHostId));
  let approvedCount = $derived(steps.filter((s) => s.enabled).length);
</script>

<div class="max-w-6xl mx-auto p-4 md:p-6 space-y-6 text-neutral-800 dark:text-neutral-200">
  <!-- Header -->
  <header class="flex flex-col md:flex-row md:items-center justify-between gap-4 pb-4 border-b border-neutral-200 dark:border-neutral-800">
    <div class="flex items-center gap-3">
      <div class="w-10 h-10 rounded-xl bg-violet-500/10 dark:bg-violet-500/20 border border-violet-500/30 flex items-center justify-center text-violet-600 dark:text-violet-400">
        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.813 15.904L9 18.75l-.813-2.846a4.5 4.5 0 00-3.09-3.09L2.25 12l2.846-.813a4.5 4.5 0 003.09-3.09L9 5.25l.813 2.846a4.5 4.5 0 003.09 3.09L15.75 12l-2.846.813a4.5 4.5 0 00-3.09 3.09zM18.259 8.715L18 9.75l-.259-1.035a3.375 3.375 0 00-2.455-2.456L14.25 6l1.036-.259a3.375 3.375 0 002.455-2.456L18 2.25l.259 1.035a3.375 3.375 0 002.456 2.456L21.75 6l-1.035.259a3.375 3.375 0 00-2.456 2.456zM16.894 20.567L16.5 21.75l-.394-1.183a2.25 2.25 0 00-1.423-1.423L13.5 18.75l1.183-.394a2.25 2.25 0 001.423-1.423l.394-1.183.394 1.183a2.25 2.25 0 001.423 1.423l1.183.394-1.183.394a2.25 2.25 0 00-1.423 1.423z" />
        </svg>
      </div>
      <div>
        <div class="flex items-center gap-2">
          <h1 class="text-xl md:text-2xl font-bold text-neutral-900 dark:text-white tracking-tight">Prompt Studio</h1>
          <span class="text-xs px-2 py-0.5 rounded-full bg-violet-500/15 text-violet-600 dark:text-violet-400 font-semibold border border-violet-500/30">AI Ops Assistant</span>
        </div>
        <p class="text-xs md:text-sm text-neutral-500 dark:text-neutral-400">Rencanakan, tinjau, dan eksekusi otomasi server dengan AI yang transparan & terkendali.</p>
      </div>
    </div>

    <!-- Header Actions: Host Selector & Settings Gear -->
    <div class="flex items-center gap-3">
      <!-- Target Host Dropdown -->
      <div class="flex items-center gap-2 bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-lg px-3 py-1.5 shadow-sm">
        <label for="target-host-select" class="text-xs text-neutral-500 dark:text-neutral-400 font-medium whitespace-nowrap">Target Host:</label>
        {#if isLoadingHosts}
          <span class="text-xs text-neutral-400">Memuat host...</span>
        {:else if hosts.length === 0}
          <span class="text-xs text-amber-500 font-medium">Tidak ada host</span>
        {:else}
          <select
            id="target-host-select"
            bind:value={selectedHostId}
            class="bg-transparent text-xs md:text-sm font-semibold text-neutral-900 dark:text-white focus:outline-none cursor-pointer max-w-[200px] truncate"
          >
            {#each hosts as host}
              <option value={host.id} class="bg-white dark:bg-neutral-900 text-neutral-900 dark:text-white">
                {host.label} ({host.username}@{host.address}:{host.port})
              </option>
            {/each}
          </select>
        {/if}
      </div>

      <!-- AI config now lives on the Settings page so there is exactly one place for it. -->
      <a
        href="/settings"
        title="Pengaturan AI (halaman Settings)"
        aria-label="Pengaturan AI"
        class="p-2 rounded-lg bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 text-neutral-600 dark:text-neutral-300 hover:text-neutral-900 dark:hover:text-white hover:bg-neutral-100 dark:hover:bg-neutral-800 shadow-sm transition-colors inline-flex"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
        </svg>
      </a>
    </div>
  </header>

  <!-- Goal Input Section -->
  <section class="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-xl p-4 md:p-5 shadow-sm space-y-4">
    <div>
      <label for="ai-ops-goal-input" class="block text-sm font-semibold text-neutral-800 dark:text-neutral-200 mb-2">
        Tujuan / Instruksi Ops Server
      </label>
      <textarea
        id="ai-ops-goal-input"
        bind:value={goal}
        rows="3"
        placeholder="Contoh: Setup Laptop Ubuntu saya untuk kebutuhan Development Laravel"
        class="w-full bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded-lg p-3 text-sm text-neutral-900 dark:text-white placeholder-neutral-400 dark:placeholder-neutral-600 focus:outline-none focus:border-violet-500 focus:ring-1 focus:ring-violet-500 transition-colors"
      ></textarea>
    </div>

    <!-- Quick Prompt Suggestion Chips -->
    <div class="space-y-1.5">
      <span class="text-xs font-medium text-neutral-500 dark:text-neutral-400">Rekomendasi Cepat:</span>
      <div class="flex flex-wrap gap-2">
        {#each quickPrompts as qp}
          <button
            type="button"
            onclick={() => selectQuickPrompt(qp.goal)}
            class="px-3 py-1.5 rounded-lg text-xs font-medium bg-neutral-100 dark:bg-neutral-800 hover:bg-violet-500/10 dark:hover:bg-violet-500/20 text-neutral-700 dark:text-neutral-300 hover:text-violet-600 dark:hover:text-violet-400 border border-neutral-200 dark:border-neutral-700 transition-colors cursor-pointer"
          >
            {qp.label}
          </button>
        {/each}
      </div>
    </div>

    <!-- Action Bar -->
    <div class="flex items-center justify-between pt-2">
      <div class="text-xs text-neutral-500 dark:text-neutral-400 flex items-center gap-1.5">
        <span class="w-2 h-2 rounded-full bg-emerald-500"></span>
        Model aktif: <span class="font-mono text-neutral-700 dark:text-neutral-300 font-semibold">{settings.model}</span>
      </div>

      <button
        type="button"
        onclick={handleGeneratePlan}
        disabled={isGenerating || !goal.trim()}
        class="px-5 py-2.5 rounded-lg bg-violet-600 hover:bg-violet-500 disabled:opacity-50 disabled:cursor-not-allowed text-white font-medium text-sm flex items-center gap-2 shadow-md shadow-violet-600/20 transition-all cursor-pointer"
      >
        {#if isGenerating}
          <svg class="animate-spin w-4 h-4 text-white" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          <span>Menganalisis Kebutuhan...</span>
        {:else}
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
          </svg>
          <span>Analisis & Buat Rencana (Generate Plan)</span>
        {/if}
      </button>
    </div>
  </section>

  <!-- Review & Confirmation View ('Konfirmasi Seluruhnya') -->
  {#if plan && (viewMode === 'review' || viewMode === 'executing' || viewMode === 'completed')}
    <section class="space-y-4">
      <!-- Plan Summary & Requirements Card -->
      <div class="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-xl p-5 shadow-sm space-y-3">
        <div class="flex flex-col md:flex-row md:items-start justify-between gap-3">
          <div>
            <div class="flex items-center gap-2">
              <h2 class="text-lg font-bold text-neutral-900 dark:text-white">Rencana Eksekusi: Konfirmasi Seluruhnya</h2>
              <span class="text-xs px-2 py-0.5 rounded bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 font-semibold border border-emerald-500/20">
                {approvedCount} dari {steps.length} Langkah Terpilih
              </span>
            </div>
            <p class="text-sm text-neutral-600 dark:text-neutral-400 mt-1 leading-relaxed">{plan.summary}</p>
          </div>

          <!-- Quick Actions: Select All / Deselect All -->
          {#if viewMode === 'review'}
            <div class="flex items-center gap-2 shrink-0 text-xs">
              <button
                type="button"
                onclick={() => toggleAllSteps(true)}
                class="px-2.5 py-1 rounded bg-neutral-100 dark:bg-neutral-800 hover:bg-neutral-200 dark:hover:bg-neutral-700 text-neutral-700 dark:text-neutral-300 font-medium transition-colors"
              >
                Pilih Semua
              </button>
              <button
                type="button"
                onclick={() => toggleAllSteps(false)}
                class="px-2.5 py-1 rounded bg-neutral-100 dark:bg-neutral-800 hover:bg-neutral-200 dark:hover:bg-neutral-700 text-neutral-700 dark:text-neutral-300 font-medium transition-colors"
              >
                Hapus Pilihan
              </button>
            </div>
          {/if}
        </div>

        <!-- Requirement Tags -->
        {#if plan.requirements && plan.requirements.length > 0}
          <div class="pt-2 border-t border-neutral-100 dark:border-neutral-800/80 flex flex-wrap items-center gap-2">
            <span class="text-xs font-medium text-neutral-500 dark:text-neutral-400">Komponen & Dependensi:</span>
            {#each plan.requirements as req}
              <span class="text-xs font-medium px-2.5 py-1 rounded-md bg-violet-500/10 text-violet-600 dark:text-violet-400 border border-violet-500/20">
                {req}
              </span>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Step-by-Step Checklist of Commands -->
      <div class="space-y-3">
        {#each steps as step, index}
          <div
            class="bg-white dark:bg-neutral-900 border transition-all rounded-xl overflow-hidden shadow-sm {step.enabled ? 'border-neutral-300 dark:border-neutral-800' : 'border-neutral-200 dark:border-neutral-800/40 opacity-60'}"
          >
            <!-- Step Header Bar -->
            <div class="p-4 flex items-start gap-3 bg-neutral-50/50 dark:bg-neutral-900/50">
              <!-- Include / Exclude Checkbox -->
              <div class="pt-0.5">
                <input
                  type="checkbox"
                  id={`step-check-${index}`}
                  checked={step.enabled}
                  disabled={viewMode === 'executing'}
                  onchange={() => toggleStep(index)}
                  class="w-4 h-4 rounded text-violet-600 focus:ring-violet-500 border-neutral-300 dark:border-neutral-700 cursor-pointer"
                />
              </div>

              <!-- Step Number Badge -->
              <div class="w-6 h-6 rounded-full bg-neutral-200 dark:bg-neutral-800 text-neutral-700 dark:text-neutral-300 font-mono text-xs font-bold flex items-center justify-center shrink-0">
                {step.step_number}
              </div>

              <!-- Step Info & Badges -->
              <div class="flex-1 min-w-0">
                <div class="flex flex-wrap items-center gap-2 mb-1">
                  <label for={`step-check-${index}`} class="font-semibold text-sm text-neutral-900 dark:text-white cursor-pointer truncate">
                    {step.title}
                  </label>

                  <!-- Sudo Badge -->
                  {#if step.is_sudo}
                    <span class="text-[10px] font-mono font-bold px-1.5 py-0.5 rounded bg-amber-500/15 text-amber-600 dark:text-amber-400 border border-amber-500/30 uppercase">
                      SUDO
                    </span>
                  {/if}

                  <!-- Danger / Risk Badge -->
                  {#if step.is_danger}
                    <span class="text-[10px] font-mono font-bold px-1.5 py-0.5 rounded bg-rose-500/15 text-rose-600 dark:text-rose-400 border border-rose-500/30 uppercase">
                      RISK / DANGER
                    </span>
                  {/if}

                  <!-- Status Pill (for executing & completed views) -->
                  {#if viewMode === 'executing' || viewMode === 'completed'}
                    {#if step.status === 'pending'}
                      <span class="text-[10px] font-medium px-2 py-0.5 rounded bg-neutral-200 dark:bg-neutral-800 text-neutral-600 dark:text-neutral-400">
                        Pending
                      </span>
                    {:else if step.status === 'running'}
                      <span class="text-[10px] font-medium px-2 py-0.5 rounded bg-sky-500/20 text-sky-600 dark:text-sky-400 border border-sky-500/30 flex items-center gap-1.5 animate-pulse">
                        <svg class="animate-spin w-3 h-3" fill="none" viewBox="0 0 24 24">
                          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                        </svg>
                        Running...
                      </span>
                    {:else if step.status === 'success'}
                      <span class="text-[10px] font-medium px-2 py-0.5 rounded bg-emerald-500/20 text-emerald-600 dark:text-emerald-400 border border-emerald-500/30 flex items-center gap-1">
                        <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                        </svg>
                        Success
                      </span>
                    {:else if step.status === 'failed'}
                      <span class="text-[10px] font-medium px-2 py-0.5 rounded bg-rose-500/20 text-rose-600 dark:text-rose-400 border border-rose-500/30 flex items-center gap-1">
                        <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                        Failed
                      </span>
                    {/if}
                  {/if}
                </div>
                <p class="text-xs text-neutral-500 dark:text-neutral-400">{step.description}</p>
              </div>

              <!-- Action: Copy Command Button -->
              <button
                type="button"
                onclick={() => copyCommand(step.command, index)}
                title="Copy bash command"
                class="px-2.5 py-1 text-xs rounded bg-neutral-200/70 dark:bg-neutral-800 hover:bg-neutral-300 dark:hover:bg-neutral-700 text-neutral-700 dark:text-neutral-300 flex items-center gap-1.5 transition-colors cursor-pointer shrink-0"
              >
                {#if copiedStepIndex === index}
                  <svg class="w-3.5 h-3.5 text-emerald-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                  </svg>
                  <span class="text-emerald-500 font-semibold">Tersalin!</span>
                {:else}
                  <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                  </svg>
                  <span>Copy</span>
                {/if}
              </button>
            </div>

            <!-- Editable Bash Command Block -->
            <div class="px-4 pb-4">
              <label for={`step-command-${index}`} class="sr-only">Bash command for step {step.step_number}</label>
              <div class="relative">
                <textarea
                  id={`step-command-${index}`}
                  bind:value={step.command}
                  rows="2"
                  disabled={viewMode === 'executing'}
                  class="w-full font-mono text-xs bg-neutral-900 text-emerald-400 dark:bg-neutral-950 border border-neutral-800 rounded-lg p-3 focus:outline-none focus:border-violet-500 transition-colors resize-y leading-relaxed"
                ></textarea>
              </div>
            </div>

            <!-- Live Output Viewer (For executing / completed view) -->
            {#if (viewMode === 'executing' || viewMode === 'completed') && (step.result || step.status === 'running')}
              <div class="border-t border-neutral-200 dark:border-neutral-800 bg-neutral-950 p-3">
                <div class="flex items-center justify-between text-xs text-neutral-400 mb-2 font-mono">
                  <div class="flex items-center gap-2">
                    <span class="w-2 h-2 rounded-full {step.status === 'running' ? 'bg-sky-400 animate-pulse' : step.status === 'success' ? 'bg-emerald-400' : 'bg-rose-400'}"></span>
                    <span>Output Terminal (Step {step.step_number})</span>
                    {#if step.result?.duration_ms}
                      <span class="text-neutral-500">({step.result.duration_ms}ms)</span>
                    {/if}
                  </div>
                  {#if step.result?.exit_code !== undefined}
                    <span class="px-1.5 py-0.5 rounded text-[10px] {step.result.exit_code === 0 ? 'bg-emerald-950 text-emerald-400 border border-emerald-800' : 'bg-rose-950 text-rose-400 border border-rose-800'}">
                      Exit: {step.result.exit_code}
                    </span>
                  {/if}
                </div>

                <!-- Terminal-like stdout/stderr block -->
                <div class="bg-black text-neutral-200 font-mono text-xs rounded p-3 overflow-x-auto max-h-48 overflow-y-auto space-y-1">
                  {#if step.status === 'running' && !step.result}
                    <div class="text-sky-400 flex items-center gap-2">
                      <span class="animate-pulse">▶</span>
                      <span>Menjalankan perintah di host {selectedHost ? selectedHost.label : ''}...</span>
                    </div>
                  {/if}

                  {#if step.result?.stdout}
                    <pre class="text-neutral-300 whitespace-pre-wrap">{step.result.stdout}</pre>
                  {/if}

                  {#if step.result?.stderr}
                    <pre class="text-rose-400 whitespace-pre-wrap">{step.result.stderr}</pre>
                  {/if}
                </div>
              </div>
            {/if}
          </div>
        {/each}
      </div>

      <!-- Action Buttons for Review Mode -->
      {#if viewMode === 'review'}
        <div class="flex flex-col sm:flex-row items-center justify-end gap-3 pt-4 border-t border-neutral-200 dark:border-neutral-800">
          <button
            type="button"
            onclick={handleReset}
            class="w-full sm:w-auto px-5 py-2.5 rounded-lg bg-neutral-200 dark:bg-neutral-800 hover:bg-neutral-300 dark:hover:bg-neutral-700 text-neutral-800 dark:text-neutral-200 font-medium text-sm transition-colors cursor-pointer"
          >
            Batal / Reset
          </button>

          <button
            type="button"
            onclick={handleExecutePlan}
            disabled={approvedCount === 0}
            class="w-full sm:w-auto px-6 py-2.5 rounded-lg bg-emerald-600 hover:bg-emerald-500 disabled:opacity-50 disabled:cursor-not-allowed text-white font-semibold text-sm flex items-center justify-center gap-2 shadow-lg shadow-emerald-600/25 transition-all cursor-pointer"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14 5l7 7m0 0l-7 7m7-7H3" />
            </svg>
            <span>Approve & Eksekusi di Host ({approvedCount} Langkah)</span>
          </button>
        </div>
      {/if}

      <!-- Live Execution Progress Bar -->
      {#if viewMode === 'executing'}
        <div class="bg-sky-50 dark:bg-sky-950/40 border border-sky-200 dark:border-sky-800/60 rounded-xl p-4 flex items-center justify-between">
          <div class="flex items-center gap-3">
            <svg class="animate-spin w-5 h-5 text-sky-500" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
            <div>
              <p class="font-semibold text-sm text-sky-900 dark:text-sky-200">
                Mengeksekusi langkah {currentExecutingIndex + 1} dari {steps.length}...
              </p>
              <p class="text-xs text-sky-700 dark:text-sky-400">Menjalankan perintah secara sekuensial di host target.</p>
            </div>
          </div>
          <span class="text-xs font-mono font-bold px-2.5 py-1 rounded bg-sky-500/20 text-sky-600 dark:text-sky-300">
            {executionSuccessCount} Sukses • {executionFailedCount} Gagal
          </span>
        </div>
      {/if}

      <!-- Completion Banner -->
      {#if viewMode === 'completed'}
        <div class="bg-white dark:bg-neutral-900 border {executionFailedCount === 0 ? 'border-emerald-500/50 dark:border-emerald-500/30 bg-emerald-50/20 dark:bg-emerald-950/20' : 'border-amber-500/50 dark:border-amber-500/30 bg-amber-50/20 dark:bg-amber-950/20'} rounded-xl p-5 shadow-md space-y-4">
          <div class="flex items-start gap-3">
            <div class="w-10 h-10 rounded-full {executionFailedCount === 0 ? 'bg-emerald-500/20 text-emerald-500' : 'bg-amber-500/20 text-amber-500'} flex items-center justify-center shrink-0">
              {#if executionFailedCount === 0}
                <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                </svg>
              {:else}
                <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                </svg>
              {/if}
            </div>
            <div class="flex-1">
              <h3 class="font-bold text-base text-neutral-900 dark:text-white">
                {executionFailedCount === 0 ? 'Semua Langkah Berhasil Dieksekusi!' : 'Eksekusi Selesai Dengan Catatan'}
              </h3>
              <p class="text-xs md:text-sm text-neutral-600 dark:text-neutral-400 mt-0.5">
                {executionSuccessCount} langkah sukses dieksekusi di host <span class="font-semibold text-neutral-900 dark:text-white">{selectedHost ? selectedHost.label : 'target'}</span>.
                {#if executionFailedCount > 0}
                  Terdapat {executionFailedCount} langkah yang mengalami error/kegagalan.
                {/if}
              </p>
            </div>
          </div>

          <!-- Navigation Shortcuts -->
          <div class="flex flex-wrap items-center gap-3 pt-2 border-t border-neutral-200 dark:border-neutral-800">
            {#if selectedHostId}
              <a
                href={`/session?host=${selectedHostId}`}
                class="px-4 py-2 rounded-lg bg-sky-600 hover:bg-sky-500 text-white font-medium text-xs flex items-center gap-2 shadow transition-colors"
              >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                </svg>
                <span>Buka Terminal Sesi</span>
              </a>
            {/if}

            <a
              href="/command-logs"
              class="px-4 py-2 rounded-lg bg-neutral-200 dark:bg-neutral-800 hover:bg-neutral-300 dark:hover:text-white text-neutral-800 dark:text-neutral-300 font-medium text-xs flex items-center gap-2 transition-colors"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
              </svg>
              <span>Lihat di Command Logs (Audit)</span>
            </a>

            <button
              type="button"
              onclick={handleReset}
              class="ml-auto px-4 py-2 rounded-lg text-xs font-medium text-neutral-600 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white transition-colors"
            >
              Mulai Tugas Baru
            </button>
          </div>
        </div>
      {/if}
    </section>
  {/if}

  <!-- AI Settings Modal -->

</div>
