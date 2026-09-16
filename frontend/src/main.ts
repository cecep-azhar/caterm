import './style.css';
import '@xterm/xterm/css/xterm.css';

import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { IsInitialized, Setup, Unlock } from '../wailsjs/go/main/App';

const appDiv = document.querySelector('#app')!;

function renderSetup() {
  appDiv.innerHTML = `
    <div class="flex items-center justify-center h-screen w-screen bg-gray-100 dark:bg-gray-900">
      <div class="p-8 bg-white dark:bg-gray-800 rounded shadow-md w-96">
        <h2 class="text-2xl font-bold mb-4">Setup Master Password</h2>
        <input type="password" id="password" class="w-full p-2 border rounded mb-4 text-black" placeholder="Password (min 12 chars)" />
        <div class="mb-4">
          <label class="flex items-center space-x-2">
            <input type="checkbox" id="acknowledge" />
            <span class="text-sm">I acknowledge that if I lose this password, there is NO recovery.</span>
          </label>
        </div>
        <button id="setupBtn" class="w-full bg-blue-500 text-white p-2 rounded disabled:opacity-50" disabled>Setup Vault</button>
        <div id="errorMsg" class="text-red-500 mt-2 text-sm hidden"></div>
      </div>
    </div>
  `;

  const pwInput = document.getElementById('password') as HTMLInputElement;
  const ackCheck = document.getElementById('acknowledge') as HTMLInputElement;
  const setupBtn = document.getElementById('setupBtn') as HTMLButtonElement;
  const errorMsg = document.getElementById('errorMsg') as HTMLDivElement;

  ackCheck.addEventListener('change', () => {
    setupBtn.disabled = !ackCheck.checked;
  });

  setupBtn.addEventListener('click', async () => {
    const pwd = pwInput.value;
    if (pwd.length < 12) {
      errorMsg.textContent = "Password must be at least 12 characters";
      errorMsg.classList.remove('hidden');
      return;
    }
    
    try {
      await Setup(pwd);
      renderDashboard();
    } catch (e: any) {
      errorMsg.textContent = e;
      errorMsg.classList.remove('hidden');
    }
  });
}

function renderUnlock() {
  appDiv.innerHTML = `
    <div class="flex items-center justify-center h-screen w-screen bg-gray-100 dark:bg-gray-900">
      <div id="lockContainer" class="p-8 bg-white dark:bg-gray-800 rounded shadow-md w-96 transition-transform">
        <h2 class="text-2xl font-bold mb-4">Unlock Vault</h2>
        <input type="password" id="password" class="w-full p-2 border rounded mb-4 text-black" placeholder="Master Password" />
        <button id="unlockBtn" class="w-full bg-blue-500 text-white p-2 rounded">Unlock</button>
        <div id="errorMsg" class="text-red-500 mt-2 text-sm hidden"></div>
      </div>
    </div>
  `;

  const pwInput = document.getElementById('password') as HTMLInputElement;
  const unlockBtn = document.getElementById('unlockBtn') as HTMLButtonElement;
  const errorMsg = document.getElementById('errorMsg') as HTMLDivElement;
  const lockContainer = document.getElementById('lockContainer') as HTMLDivElement;

  const shake = () => {
    lockContainer.animate([
      { transform: 'translateX(0)' },
      { transform: 'translateX(-10px)' },
      { transform: 'translateX(10px)' },
      { transform: 'translateX(-10px)' },
      { transform: 'translateX(10px)' },
      { transform: 'translateX(0)' }
    ], { duration: 400 });
  };

  unlockBtn.addEventListener('click', async () => {
    try {
      await Unlock(pwInput.value);
      renderDashboard();
    } catch (e: any) {
      errorMsg.textContent = e;
      errorMsg.classList.remove('hidden');
      shake();
    }
  });

  pwInput.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') unlockBtn.click();
  });
}

function renderDashboard() {
  appDiv.innerHTML = `
    <div class="flex h-screen w-screen overflow-hidden">
      <!-- Sidebar -->
      <div class="w-64 flex-shrink-0 border-r border-gray-200 dark:border-gray-700" style="background-color: var(--sidebar-bg)">
        <div class="p-4">
          <h2 class="text-lg font-semibold">CATERM</h2>
          <ul class="mt-4 space-y-2">
            <li class="p-2 bg-gray-200 dark:bg-gray-800 rounded">Groups</li>
            <li class="p-2 hover:bg-gray-200 dark:hover:bg-gray-800 rounded cursor-pointer">Hosts</li>
            <li class="p-2 hover:bg-gray-200 dark:hover:bg-gray-800 rounded cursor-pointer">Snippets</li>
          </ul>
        </div>
      </div>

      <!-- Main Content -->
      <div class="flex-1 flex flex-col min-w-0">
        <!-- Topbar -->
        <div class="h-12 border-b border-gray-200 dark:border-gray-700 flex items-center px-4 space-x-2" style="background-color: var(--topbar-bg)">
          <div class="px-3 py-1 bg-white dark:bg-gray-900 rounded shadow-sm text-sm border border-gray-200 dark:border-gray-600">
            Terminal 1
          </div>
        </div>

        <!-- Terminal Area -->
        <div class="flex-1 relative bg-black p-2">
          <div id="terminal-container" class="absolute inset-0 p-2"></div>
        </div>
      </div>
    </div>
  `;

  const termContainer = document.getElementById('terminal-container')!;
  const term = new Terminal({
    scrollback: 10000,
    theme: { background: '#000000', foreground: '#ffffff' }
  });

  const fitAddon = new FitAddon();
  term.loadAddon(fitAddon);
  term.open(termContainer);
  fitAddon.fit();

  term.writeln('Welcome to CATERM Shell UI Demo!');
  term.write('$ ');

  window.addEventListener('resize', () => {
    fitAddon.fit();
  });
}

async function init() {
  try {
    const initialized = await IsInitialized();
    if (initialized) {
      renderUnlock();
    } else {
      renderSetup();
    }
  } catch (e) {
    console.error("Initialization error:", e);
  }
}

init();
