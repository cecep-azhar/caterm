import './style.css';
import '@xterm/xterm/css/xterm.css';

import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';

document.querySelector('#app')!.innerHTML = `
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

// Initialize Terminal
const termContainer = document.getElementById('terminal-container')!;
const term = new Terminal({
  scrollback: 10000,
  theme: {
    background: '#000000',
    foreground: '#ffffff'
  }
});

const fitAddon = new FitAddon();
term.loadAddon(fitAddon);

term.open(termContainer);
fitAddon.fit();

term.writeln('Welcome to CATERM Shell UI Demo!');
term.writeln('This is a static text rendering to demonstrate xterm.js integration.');
term.writeln('');
term.write('$ ');

// Handle resize
window.addEventListener('resize', () => {
  fitAddon.fit();
  console.log(`Terminal resized: ${term.cols} cols x ${term.rows} rows`);
});

// Initial log
console.log(`Initial terminal size: ${term.cols} cols x ${term.rows} rows`);
