import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import '@xterm/xterm/css/xterm.css';
import './style.css';
import { EventsOn, EventsOff } from "../wailsjs/runtime/runtime";
import { IsInitialized, Setup, Unlock, CheckSyncPending, ApplySync, ListHosts, CreateHost, UpdateHost, DeleteHost, ListGroups, CreateGroup, DeleteGroup, ResetVault, ConnectTerminal, WriteTerminal, CloseTerminal } from "../wailsjs/go/main/App";

let activeTab = "hosts";
let editingHostId: string | null = null;
let sessionTimerInterval: any = null;
let sessionSeconds = 0;

document.addEventListener("DOMContentLoaded", async () => {
    const app = document.getElementById("app");
    if (!app) return;

    try {
        const isInit = await IsInitialized();
        if (isInit) {
            const syncResult = await CheckSyncPending();
            if (syncResult && (syncResult.added > 0 || syncResult.updated > 0 || syncResult.deleted > 0 || (syncResult.conflicts && syncResult.conflicts.length > 0))) {
                renderSyncReview(app, syncResult);
            } else {
                renderUnlock(app);
            }
        } else {
            renderSetup(app);
        }
    } catch (e) {
        console.error("Failed to check initialization status:", e);
        app.innerHTML = `<div class="min-h-screen flex items-center justify-center bg-[#0d1117]"><div class="text-[#f85149] p-6 rounded-xl border border-[#f85149]">Error: ${e}</div></div>`;
    }
});

function setupPasswordToggles(container: HTMLElement) {
    container.querySelectorAll('.toggle-pwd').forEach(btn => {
        btn.addEventListener('click', () => {
            const targetId = (btn as HTMLElement).getAttribute('data-target');
            if (!targetId) return;
            const input = document.getElementById(targetId) as HTMLInputElement;
            if (!input) return;

            if (input.type === 'password') {
                input.type = 'text';
                btn.innerHTML = `<svg class="h-5 w-5 eye-icon text-[#58a6ff]" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858-5.908a8.959 8.959 0 013.682-.793c4.478 0 8.268 2.943 9.542 7a10.025 10.025 0 01-4.132 5.411m-1.92-2.5a3 3 0 00-4.153-4.155m-3.484 3.483a3 3 0 004.155 4.153M3 3l18 18"/></svg>`;
            } else {
                input.type = 'password';
                btn.innerHTML = `<svg class="h-5 w-5 eye-icon" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"/></svg>`;
            }
        });
    });
}

function renderSetup(container: HTMLElement) {
    container.innerHTML = `
        <div class="min-h-screen flex bg-[#0d1117] text-[#e6edf3] font-sans">
            <div class="flex-1 flex flex-col justify-between p-12 bg-[#010409] border-r border-[#30363d]">
                <div>
                    <div class="flex items-center space-x-3 mb-2">
                        <div class="h-10 w-10 rounded-xl bg-[#0b0f19] flex items-center justify-center font-mono font-bold text-[#38bdf8] shadow-lg text-lg tracking-wider border border-[#38bdf8]/30">&gt;_</div>
                        <h1 class="text-2xl font-bold tracking-tight text-white">CATerm</h1>
                    </div>
                </div>
                <div>
                    <h2 class="text-4xl font-bold mb-4 text-[#e6edf3]">// KEEP CALM - When stuck, tail -f the logs</h2>
                </div>
                <div class="text-[#8b949e] text-sm flex items-center space-x-2">
                    <span class="inline-block h-2 w-2 rounded-full bg-emerald-500"></span>
                    <span>Zero-knowledge · keys never leave this device</span>
                </div>
            </div>

            <div class="w-[480px] flex flex-col justify-center p-12 bg-[#0d1117]">
                <div class="mb-8">
                    <h2 class="text-3xl font-bold mb-2">Setup Vault</h2>
                    <p class="text-[#8b949e]">cecep.azhtech@gmail.com</p>
                </div>
                
                <form id="setup-form" class="space-y-5">
                    <div>
                        <label class="block text-sm font-medium mb-2 text-[#e6edf3]">Master Password (min 8 chars)</label>
                        <div class="relative">
                            <input type="password" id="setup-password" class="w-full p-3 pr-10 rounded-md custom-input" placeholder="Enter master password" required>
                            <button type="button" class="toggle-pwd absolute inset-y-0 right-0 pr-3 flex items-center text-[#8b949e] hover:text-white" data-target="setup-password">
                                <svg class="h-5 w-5 eye-icon" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"/></svg>
                            </button>
                        </div>
                        <p id="setup-error" class="text-[#f85149] text-sm hidden mt-2"></p>
                    </div>
                    <div>
                        <label class="block text-sm font-medium mb-2 text-[#e6edf3]">Confirm Password</label>
                        <div class="relative">
                            <input type="password" id="setup-confirm" class="w-full p-3 pr-10 rounded-md custom-input" placeholder="Confirm master password" required>
                            <button type="button" class="toggle-pwd absolute inset-y-0 right-0 pr-3 flex items-center text-[#8b949e] hover:text-white" data-target="setup-confirm">
                                <svg class="h-5 w-5 eye-icon" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"/></svg>
                            </button>
                        </div>
                    </div>
                    <button type="submit" class="w-full bg-[#238636] hover:bg-[#2ea043] text-white font-semibold py-3 px-4 rounded-md transition-colors shadow-lg">Initialize Vault</button>
                </form>
            </div>
        </div>
    `;

    setupPasswordToggles(container);

    const form = document.getElementById("setup-form") as HTMLFormElement;
    const pwdInput = document.getElementById("setup-password") as HTMLInputElement;
    const confirmInput = document.getElementById("setup-confirm") as HTMLInputElement;
    const errorText = document.getElementById("setup-error") as HTMLParagraphElement;

    form.addEventListener("submit", async (e) => {
        e.preventDefault();
        const pwd = pwdInput.value;
        if (!pwd || pwd.length < 8) {
            errorText.textContent = "Master Password must be at least 8 characters.";
            errorText.classList.remove("hidden");
            return;
        }
        if (pwd !== confirmInput.value) {
            errorText.textContent = "Passwords do not match.";
            errorText.classList.remove("hidden");
            return;
        }

        try {
            await Setup(pwd);
            renderUnlock(container);
        } catch (err: any) {
            errorText.textContent = err.toString();
            errorText.classList.remove("hidden");
        }
    });
}

function renderUnlock(container: HTMLElement) {
    container.innerHTML = `
        <div id="unlock-container" class="min-h-screen flex bg-[#0d1117] text-[#e6edf3] font-sans">
            <div class="flex-1 flex flex-col justify-between p-12 bg-[#010409] border-r border-[#30363d]">
                <div>
                    <div class="flex items-center space-x-3 mb-2">
                        <div class="h-10 w-10 rounded-xl bg-[#0b0f19] flex items-center justify-center font-mono font-bold text-[#38bdf8] shadow-lg text-lg tracking-wider border border-[#38bdf8]/30">&gt;_</div>
                        <h1 class="text-2xl font-bold tracking-tight text-white">CATerm</h1>
                    </div>
                </div>
                <div>
                    <h2 class="text-4xl font-bold mb-4 text-[#e6edf3]">// KEEP CALM - When stuck, tail -f the logs</h2>
                </div>
                <div class="text-[#8b949e] text-sm flex items-center space-x-2">
                    <span class="inline-block h-2 w-2 rounded-full bg-emerald-500"></span>
                    <span>Zero-knowledge · keys never leave this device</span>
                </div>
            </div>

            <div class="w-[480px] flex flex-col justify-center p-12 bg-[#0d1117]">
                <div class="mb-8">
                    <h2 class="text-3xl font-bold mb-2">Welcome back</h2>
                    <p class="text-[#8b949e]">cecep.azhtech@gmail.com</p>
                </div>
                
                <form id="unlock-form" class="space-y-6" autocomplete="off">
                    <div>
                        <label class="block text-sm font-medium mb-2 text-[#e6edf3]">Master Password</label>
                        <div class="relative">
                            <input type="password" id="unlock-password" class="w-full p-3 pr-10 rounded-md custom-input" placeholder="Enter master password" required autocomplete="current-password">
                            <button type="button" class="toggle-pwd absolute inset-y-0 right-0 pr-3 flex items-center text-[#8b949e] hover:text-white" data-target="unlock-password">
                                <svg class="h-5 w-5 eye-icon" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"/></svg>
                            </button>
                        </div>
                        <p id="unlock-error" class="text-[#f85149] text-sm hidden mt-2"></p>
                    </div>
                    <button type="submit" class="w-full bg-[#238636] hover:bg-[#2ea043] text-white font-semibold py-3 px-4 rounded-md transition-colors shadow-lg">Unlock</button>
                    <button type="button" id="btn-reset-vault" class="w-full mt-2 bg-transparent hover:bg-rose-500/10 text-rose-500 border border-rose-500/20 font-semibold py-2 px-4 rounded-md transition-colors text-sm">Reset Vault</button>
                </form>
            </div>
        </div>
    `;

    setupPasswordToggles(container);

    const form = document.getElementById("unlock-form") as HTMLFormElement;
    const pwdInput = document.getElementById("unlock-password") as HTMLInputElement;
    const errorText = document.getElementById("unlock-error") as HTMLParagraphElement;

    errorText.classList.add("hidden");
    errorText.textContent = "";

    setTimeout(() => pwdInput.focus(), 100);

    form.addEventListener("submit", async (e) => {
        e.preventDefault();
        const pwd = pwdInput.value;
        if (!pwd || pwd.trim() === "") return;
        
        const submitBtn = form.querySelector('button[type="submit"]') as HTMLButtonElement;
        const originalText = submitBtn.textContent;
        submitBtn.textContent = "Unlocking...";
        submitBtn.disabled = true;

        try {
            await Unlock(pwd);
            renderDashboard(container);
        } catch (err: any) {
            errorText.textContent = "Incorrect password.";
            errorText.classList.remove("hidden");
            pwdInput.classList.add("border-[#f85149]", "focus:border-[#f85149]", "focus:ring-[#f85149]/20");
            
            submitBtn.textContent = originalText || "Unlock";
            submitBtn.disabled = false;
            
            pwdInput.value = "";
            pwdInput.focus();
        }
    });
    
    pwdInput.addEventListener('input', () => {
        pwdInput.classList.remove("border-[#f85149]", "focus:border-[#f85149]", "focus:ring-[#f85149]/20");
        errorText.classList.add("hidden");
        errorText.textContent = "";
    });

    document.getElementById("btn-reset-vault")?.addEventListener("click", async () => {
        if (confirm("Reset Vault will permanently delete all saved hosts and settings. Continue?")) {
            try {
                await ResetVault();
                renderSetup(container);
            } catch (e: any) {
                alert("Failed to reset vault: " + e.toString());
            }
        }
    });
}

function renderSyncReview(container: HTMLElement, syncResult: any) {
    container.innerHTML = `
        <div class="min-h-screen flex items-center justify-center bg-[#0d1117] text-[#e6edf3]">
            <div class="bg-[#161b22] p-8 rounded-xl border border-[#30363d] w-[450px]">
                <h2 class="text-2xl font-bold mb-6 text-[#58a6ff]">Sync Pending</h2>
                <div class="space-y-4 mb-8 bg-[#0d1117] p-4 rounded-lg border border-[#30363d]">
                    <div class="flex justify-between items-center pb-2 border-b border-[#30363d]">
                        <span class="text-[#8b949e]">Added changes:</span>
                        <span class="font-bold text-[#3fb950]">${syncResult.added || 0}</span>
                    </div>
                    <div class="flex justify-between items-center pb-2 border-b border-[#30363d]">
                        <span class="text-[#8b949e]">Updated changes:</span>
                        <span class="font-bold text-[#38bdf8]">${syncResult.updated || 0}</span>
                    </div>
                    <div class="flex justify-between items-center pb-2 border-b border-[#30363d]">
                        <span class="text-[#8b949e]">Deleted records:</span>
                        <span class="font-bold text-[#f85149]">${syncResult.deleted || 0}</span>
                    </div>
                    <div class="flex justify-between items-center">
                        <span class="text-[#8b949e]">Conflicts:</span>
                        <span class="font-bold text-[#d29922]">${syncResult.conflicts ? syncResult.conflicts.length : 0}</span>
                    </div>
                </div>
                <div class="flex space-x-4">
                    <button id="sync-cancel" class="flex-1 bg-[#21262d] hover:bg-[#30363d] text-[#e6edf3] font-semibold py-2.5 px-4 rounded-md border border-[#30363d]">Skip</button>
                    <button id="sync-apply" class="flex-1 bg-[#238636] hover:bg-[#2ea043] text-white font-semibold py-2.5 px-4 rounded-md">Apply Sync</button>
                </div>
            </div>
        </div>
    `;

    document.getElementById("sync-cancel")?.addEventListener("click", () => {
        renderUnlock(container);
    });

    document.getElementById("sync-apply")?.addEventListener("click", async () => {
        try {
            await ApplySync();
            renderUnlock(container);
        } catch (e: any) {
            alert("Failed to apply sync: " + e.toString());
        }
    });
}

async function renderDashboard(container: HTMLElement) {
    let hosts: any[] = [];
    let groups: any[] = [];
    try {
        hosts = await ListHosts() || [];
        groups = await ListGroups() || [];
    } catch (e) {
        console.error("Failed to load initial data", e);
    }

    const menuItems = [
        { id: "hosts", label: "Hosts", icon: '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M12 5l7 7-7 7"/>' },
        { id: "groups", label: "Groups", icon: '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 002-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"/>' },
        { id: "snippets", label: "Snippets", icon: '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>' },
        { id: "teams", label: "Teams", icon: '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"/>' },
        { id: "port_forwarding", label: "Port forwarding", icon: '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4"/>' },
        { id: "monitoring", label: "Monitoring", icon: '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"/>' },
        { id: "command_logs", label: "Command logs", icon: '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"/>' },
        { id: "ssh_keys", label: "SSH keys", icon: '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z"/>' }
    ];

    container.innerHTML = `
        <div class="h-screen flex bg-[#0d1117] text-[#e6edf3] font-sans overflow-hidden">
            <div class="w-64 flex flex-col bg-[#010409] border-r border-[#30363d]">
                <div class="p-4 border-b border-[#30363d] flex items-center space-x-3">
                    <div class="h-8 w-8 rounded-lg bg-[#0b0f19] flex items-center justify-center font-mono font-bold text-[#38bdf8] text-xs shadow-md border border-[#38bdf8]/30">&gt;_</div>
                    <h1 class="text-xl font-bold tracking-tight text-white">CATerm</h1>
                </div>
                
                <div class="flex-1 overflow-y-auto py-4">
                    <nav class="space-y-1 px-2" id="sidebar-nav">
                        ${menuItems.map(item => `
                            <a href="#" data-tab="${item.id}" class="nav-link flex items-center px-3 py-2 rounded-md font-medium text-sm group transition-colors ${activeTab === item.id ? 'bg-[#161b22] text-[#e6edf3] border border-[#30363d]' : 'text-[#8b949e] hover:bg-[#161b22] hover:text-[#e6edf3]'}">
                                <span class="mr-3 ${activeTab === item.id ? 'text-[#e6edf3]' : 'text-[#8b949e] group-hover:text-[#e6edf3]'}">
                                    <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">${item.icon}</svg>
                                </span>
                                ${item.label}
                            </a>
                        `).join('')}
                    </nav>
                </div>
                
                <div class="p-4 border-t border-[#30363d] flex items-center group cursor-pointer hover:bg-[#161b22] transition-colors">
                    <div class="h-8 w-8 rounded-full bg-[#1e293b] border border-[#334155] flex items-center justify-center text-white font-bold mr-3 text-xs shadow-[0_0_10px_rgba(0,0,0,0.5)]">
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-sky-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
                        </svg>
                    </div>
                    <div class="flex-1 overflow-hidden">
                        <p class="text-sm font-medium text-[#e6edf3] truncate">Cecep Saeful Azhar</p>
                        <p class="text-xs text-[#8b949e] truncate">cecep.azhtech@gmail.com</p>
                    </div>
                </div>
            </div>

            <div class="flex-1 flex flex-col relative overflow-hidden" id="main-view">
                <!-- Dynamic Content -->
            </div>
        </div>
    `;

    container.querySelectorAll('.nav-link').forEach(link => {
        link.addEventListener('click', (e) => {
            e.preventDefault();
            activeTab = (link as HTMLElement).getAttribute('data-tab') || 'hosts';
            renderDashboard(container);
        });
    });

    const mainView = document.getElementById("main-view")!;

    if (activeTab === "hosts") {
        renderHostsView(mainView, container, hosts, groups);
    } else if (activeTab === "groups") {
        renderGroupsView(mainView, container, groups);
    } else {
        renderGenericView(mainView, activeTab);
    }
}

function renderHostsView(mainView: HTMLElement, container: HTMLElement, hosts: any[], groups: any[]) {
    mainView.innerHTML = `
        <div class="px-8 py-6 border-b border-[#30363d] flex justify-between items-center bg-[#0d1117]">
            <div class="flex-1 max-w-xl">
                <div class="relative">
                    <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                        <svg class="h-5 w-5 text-[#8b949e]" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/></svg>
                    </div>
                    <input type="text" id="search-hosts" placeholder="Search hosts..." class="block w-full pl-10 pr-3 py-2 border border-[#30363d] rounded-md leading-5 bg-[#0d1117] text-[#e6edf3] placeholder-[#8b949e] focus:outline-none focus:border-[#58a6ff] focus:ring-1 focus:ring-[#58a6ff] sm:text-sm transition-colors">
                </div>
            </div>
            <div class="ml-4 flex items-center">
                <span class="text-[#8b949e] text-sm mr-4">${hosts.length} saved hosts</span>
                <button id="btn-add-host" class="bg-[#238636] hover:bg-[#2ea043] text-white font-medium py-2 px-4 rounded-md text-sm flex items-center shadow-md transition-colors">
                    <svg class="h-4 w-4 mr-1.5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/></svg>
                    Add host
                </button>
            </div>
        </div>

        <div class="flex-1 overflow-y-auto p-8" id="hosts-container">
            ${hosts.length === 0 ? `
                <div class="h-full flex flex-col items-center justify-center text-center">
                    <svg class="h-12 w-12 text-[#30363d] mb-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M5 12h14M12 5l7 7-7 7"/></svg>
                    <h3 class="text-lg font-medium text-[#e6edf3]">No hosts yet</h3>
                    <p class="mt-1 text-sm text-[#8b949e]">Get started by adding your first server connection.</p>
                </div>
            ` : `
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                    ${hosts.map(h => `
                        <div class="bg-[#161b22] border border-[#30363d] rounded-md p-4 hover:border-[#8b949e] transition-colors group flex flex-col justify-between">
                            <div>
                                <div class="flex justify-between items-start mb-2">
                                    <h3 class="font-medium text-[#58a6ff] hover:underline cursor-pointer btn-connect-host" data-host-id="${h.id}">${h.name || h.hostname}</h3>
                                    <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-[#1f2328] text-[#8b949e] border border-[#30363d]">Port ${h.port}</span>
                                </div>
                                <p class="text-sm text-[#8b949e] truncate font-mono">${h.username}@${h.hostname}</p>
                            </div>
                            <div class="mt-4 pt-3 border-t border-[#30363d]/50 flex items-center justify-between">
                                <button data-connect-host="${h.id}" class="btn-connect-host bg-[#238636] hover:bg-[#2ea043] text-white text-xs font-semibold px-3 py-1.5 rounded transition-colors flex items-center">
                                    <svg class="h-3.5 w-3.5 mr-1" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"/><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/></svg>
                                    Connect
                                </button>
                                <div class="flex items-center space-x-2">
                                    <button data-edit-host="${h.id}" class="btn-edit-host text-xs text-[#58a6ff] hover:text-[#79c0ff] px-1.5 py-1">Edit</button>
                                    <button data-clone-host="${h.id}" class="btn-clone-host text-xs text-[#d29922] hover:text-[#e3b341] px-1.5 py-1">Clone</button>
                                    <button data-del-host="${h.id}" class="btn-del-host text-xs text-[#f85149] hover:text-[#ff7b72] px-1.5 py-1">Delete</button>
                                </div>
                            </div>
                        </div>
                    `).join('')}
                </div>
            `}
        </div>

        <!-- Terminal Workspace Modal with Split, Files, Time Track, Safe Workspace -->
        <div id="terminal-modal" class="fixed inset-0 bg-[#0d1117] z-50 flex flex-col hidden">
            <!-- Modal Header / Toolbar -->
            <div class="bg-[#010409] px-4 py-2 border-b border-[#30363d] flex justify-between items-center text-xs">
                <div class="flex items-center space-x-3">
                    <span class="inline-block h-2.5 w-2.5 rounded-full bg-emerald-500 animate-pulse"></span>
                    <span id="term-title" class="font-mono font-bold text-[#38bdf8]">SSH Terminal Session</span>
                    <span class="bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 px-2 py-0.5 rounded text-[10px] font-mono">🔒 Safe Workspace Active</span>
                </div>
                
                <!-- Action Tools (Split, Files, Timer, Close) -->
                <div class="flex items-center space-x-3">
                    <!-- Session Duration Tracker -->
                    <div class="flex items-center space-x-1 text-[#8b949e] font-mono bg-[#161b22] px-2.5 py-1 rounded border border-[#30363d]">
                        <svg class="h-3.5 w-3.5 text-[#38bdf8]" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"/></svg>
                        <span id="session-timer">00:00:00</span>
                    </div>

                    <!-- Split Actions -->
                    <div class="flex items-center space-x-1 bg-[#161b22] p-0.5 rounded border border-[#30363d]">
                        <button id="btn-split-v" title="Split Vertically" class="px-2 py-1 text-[#8b949e] hover:text-white hover:bg-[#30363d] rounded transition-colors flex items-center">
                            <svg class="h-3.5 w-3.5 mr-1" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 17V7m6 10V7M3 12h18"/></svg>
                            Split V
                        </button>
                        <button id="btn-split-h" title="Split Horizontally" class="px-2 py-1 text-[#8b949e] hover:text-white hover:bg-[#30363d] rounded transition-colors flex items-center">
                            <svg class="h-3.5 w-3.5 mr-1" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 3v18M3 9h18M3 15h18"/></svg>
                            Split H
                        </button>
                    </div>

                    <!-- SFTP Files Toggle -->
                    <button id="btn-toggle-sftp" title="SFTP Files Explorer" class="bg-[#161b22] text-[#8b949e] hover:text-white px-2.5 py-1 rounded border border-[#30363d] flex items-center">
                        <svg class="h-3.5 w-3.5 mr-1 text-[#58a6ff]" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"/></svg>
                        Files (SFTP)
                    </button>

                    <!-- Close Workspace -->
                    <button id="btn-close-term" class="bg-rose-500/10 text-rose-400 hover:bg-rose-500/20 px-3 py-1 rounded border border-rose-500/30 transition-colors font-medium">
                        Close Workspace
                    </button>
                </div>
            </div>

            <!-- Main Terminal Workspace Content Area -->
            <div class="flex-1 flex overflow-hidden">
                <!-- Terminal Panes Layout Grid -->
                <div class="flex-1 grid grid-cols-1 gap-1 p-2 bg-[#090d16]" id="panes-container">
                    <div class="w-full h-full border border-[#30363d] rounded p-2 overflow-hidden bg-[#0d1117] flex flex-col relative" id="pane-1">
                        <div class="text-[10px] text-[#8b949e] font-mono pb-1 border-b border-[#30363d] flex justify-between items-center">
                            <span>Pane 1 - Primary SSH Session</span>
                            <button class="hover:text-rose-400 btn-close-pane">✕ Close</button>
                        </div>
                        <div class="flex-1 w-full h-full" id="term-screen-1"></div>
                    </div>
                </div>

                <!-- SFTP Sidebar Panel (Toggleable) -->
                <div id="sftp-panel" class="w-72 bg-[#010409] border-l border-[#30363d] flex flex-col hidden">
                    <div class="p-3 border-b border-[#30363d] flex justify-between items-center">
                        <span class="font-bold text-xs text-[#58a6ff] flex items-center">
                            <svg class="h-4 w-4 mr-1" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"/></svg>
                            SFTP Remote Explorer
                        </span>
                        <button id="btn-close-sftp" class="text-[#8b949e] hover:text-white text-xs">✕</button>
                    </div>
                    <div class="p-2 border-b border-[#30363d] bg-[#0d1117]">
                        <input type="text" value="/var/www/html" class="w-full p-1.5 bg-[#161b22] border border-[#30363d] rounded text-xs font-mono text-[#e6edf3]">
                    </div>
                    <div class="flex-1 overflow-y-auto p-2 space-y-1 font-mono text-xs">
                        <div class="p-1.5 rounded hover:bg-[#161b22] cursor-pointer text-[#8b949e] flex items-center">📁 ..</div>
                        <div class="p-1.5 rounded hover:bg-[#161b22] cursor-pointer text-[#38bdf8] flex items-center">📁 config/</div>
                        <div class="p-1.5 rounded hover:bg-[#161b22] cursor-pointer text-[#38bdf8] flex items-center">📁 logs/</div>
                        <div class="p-1.5 rounded hover:bg-[#161b22] cursor-pointer text-[#e6edf3] flex items-center">📄 index.php</div>
                        <div class="p-1.5 rounded hover:bg-[#161b22] cursor-pointer text-[#e6edf3] flex items-center">📄 docker-compose.yml</div>
                        <div class="p-1.5 rounded hover:bg-[#161b22] cursor-pointer text-[#e6edf3] flex items-center">📄 .env.production</div>
                    </div>
                </div>
            </div>
        </div>

        <!-- Slide Panel (Add / Edit Host) -->
        <div id="slide-panel" class="absolute inset-y-0 right-0 w-96 bg-[#161b22] border-l border-[#30363d] transform translate-x-full transition-transform duration-300 ease-in-out z-20 shadow-2xl flex flex-col">
            <div class="px-6 py-4 border-b border-[#30363d] flex justify-between items-center bg-[#010409]">
                <h2 class="text-lg font-medium" id="panel-title">Add new host</h2>
                <button id="btn-close-panel" class="text-[#8b949e] hover:text-[#e6edf3] transition-colors">
                    <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/></svg>
                </button>
            </div>
            
            <div class="flex-1 overflow-y-auto p-6">
                <form id="add-host-form" class="space-y-5">
                    <div>
                        <label class="block text-sm font-medium mb-1 text-[#e6edf3]">Label / Name *</label>
                        <input type="text" id="host-name" required placeholder="e.g. Production Web" class="w-full p-2.5 rounded-md custom-input text-sm">
                    </div>
                    
                    <div class="flex space-x-4">
                        <div class="flex-1">
                            <label class="block text-sm font-medium mb-1 text-[#e6edf3]">Host / IP *</label>
                            <input type="text" id="host-ip" required placeholder="192.168.1.1" class="w-full p-2.5 rounded-md custom-input text-sm">
                        </div>
                        <div class="w-24">
                            <label class="block text-sm font-medium mb-1 text-[#e6edf3]">Port *</label>
                            <input type="number" id="host-port" required value="22" class="w-full p-2.5 rounded-md custom-input text-sm">
                        </div>
                    </div>
                    
                    <div>
                        <label class="block text-sm font-medium mb-1 text-[#e6edf3]">Username *</label>
                        <input type="text" id="host-user" required value="root" class="w-full p-2.5 rounded-md custom-input text-sm">
                    </div>
                    
                    <div>
                        <label class="block text-sm font-medium mb-1 text-[#e6edf3]">Authentication Method</label>
                        <select id="host-auth" class="w-full p-2.5 rounded-md custom-input text-sm">
                            <option value="password">Password</option>
                            <option value="private_key">Private Key</option>
                        </select>
                    </div>

                    <div id="auth-password-group">
                        <label class="block text-sm font-medium mb-1 text-[#e6edf3]">Password</label>
                        <div class="relative">
                            <input type="password" id="host-pass" class="w-full p-2.5 pr-10 rounded-md custom-input text-sm">
                            <button type="button" class="toggle-pwd absolute inset-y-0 right-0 pr-3 flex items-center text-[#8b949e] hover:text-white" data-target="host-pass">
                                <svg class="h-4 w-4 eye-icon" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"/></svg>
                            </button>
                        </div>
                    </div>

                    <div id="auth-key-group" class="hidden">
                        <label class="block text-sm font-medium mb-1 text-[#e6edf3]">Private Key</label>
                        <textarea id="host-key" rows="4" class="w-full p-2.5 rounded-md custom-input text-sm font-mono text-xs"></textarea>
                    </div>
                    
                    <div>
                        <label class="block text-sm font-medium mb-1 text-[#e6edf3]">Group / Organization</label>
                        <select id="host-group" class="w-full p-2.5 rounded-md custom-input text-sm">
                            <option value="">-- None --</option>
                            ${groups.map(g => `<option value="${g.id}">${g.name}</option>`).join('')}
                        </select>
                    </div>
                </form>
            </div>
            
            <div class="p-6 border-t border-[#30363d] bg-[#010409]">
                <button type="button" id="btn-save-host" class="w-full bg-[#238636] hover:bg-[#2ea043] text-white font-medium py-2.5 px-4 rounded-md text-sm transition-colors shadow-md">Save Host</button>
            </div>
        </div>
    `;

    setupPasswordToggles(mainView);

    const panel = document.getElementById("slide-panel")!;
    const panelTitle = document.getElementById("panel-title")!;
    const btnAdd = document.getElementById("btn-add-host")!;
    const btnClose = document.getElementById("btn-close-panel")!;
    
    btnAdd.addEventListener("click", () => {
        editingHostId = null;
        panelTitle.textContent = "Add new host";
        (document.getElementById("add-host-form") as HTMLFormElement).reset();
        panel.classList.remove("translate-x-full");
    });
    
    btnClose.addEventListener("click", () => panel.classList.add("translate-x-full"));

    const authSelect = document.getElementById("host-auth") as HTMLSelectElement;
    const passGroup = document.getElementById("auth-password-group")!;
    const keyGroup = document.getElementById("auth-key-group")!;

    authSelect.addEventListener("change", () => {
        if (authSelect.value === "password") {
            passGroup.classList.remove("hidden");
            keyGroup.classList.add("hidden");
        } else {
            passGroup.classList.add("hidden");
            keyGroup.classList.remove("hidden");
        }
    });

    // CONNECT handler (with interactive xterm.js, Split Panes, Timer, SFTP)
    let currentTerm: Terminal | null = null;
    document.querySelectorAll('.btn-connect-host').forEach(btn => {
        btn.addEventListener('click', (e) => {
            e.stopPropagation();
            const id = (btn as HTMLElement).getAttribute('data-connect-host') || (btn as HTMLElement).getAttribute('data-host-id');
            const h = hosts.find(item => item.id === id);
            if (!h) return;

            const modal = document.getElementById("terminal-modal")!;
            const title = document.getElementById("term-title")!;
            const screen = document.getElementById("term-screen-1")!;

            title.textContent = `SSH Workspace: ${h.username}@${h.hostname}:${h.port}`;
            modal.classList.remove("hidden");
            screen.innerHTML = "";

            // Start Session Duration Timer
            sessionSeconds = 0;
            if (sessionTimerInterval) clearInterval(sessionTimerInterval);
            sessionTimerInterval = setInterval(() => {
                sessionSeconds++;
                const hrs = String(Math.floor(sessionSeconds / 3600)).padStart(2, '0');
                const mins = String(Math.floor((sessionSeconds % 3600) / 60)).padStart(2, '0');
                const secs = String(sessionSeconds % 60).padStart(2, '0');
                const timerElem = document.getElementById("session-timer");
                if (timerElem) timerElem.textContent = `${hrs}:${mins}:${secs}`;
            }, 1000);

            if (currentTerm) {
                currentTerm.dispose();
            }

            const term = new Terminal({
                cursorBlink: true,
                theme: {
                    background: '#0d1117',
                    foreground: '#e6edf3',
                    cursor: '#58a6ff',
                }
            });

            const fitAddon = new FitAddon();
            term.loadAddon(fitAddon);
            term.open(screen);
            fitAddon.fit();

            currentTerm = term;

            const paneID = "pane-1";
            term.writeln('\x1b[32m[CATerm SSH PTY Engine v0.1.0]\x1b[0m');
            term.writeln(`\x1b[90mConnecting to ${h.username}@${h.hostname}:${h.port}...\x1b[0m`);

            // Wire up real Wails Events for SSH Output
            EventsOff(`terminal:output:${paneID}`);
            EventsOn(`terminal:output:${paneID}`, (data: string) => {
                term.write(data);
            });

            EventsOff(`terminal:closed:${paneID}`);
            EventsOn(`terminal:closed:${paneID}`, () => {
                term.writeln('\r\n\x1b[31m[Connection Closed by Remote Host]\x1b[0m');
            });

            // Trigger real Go SSH PTY Connection
            ConnectTerminal(paneID, h.id, 24, 80).catch((err: any) => {
                term.writeln(`\r\n\x1b[31m✔ SSH Connection Failed: ${err}\x1b[0m`);
            });

            term.onData(data => {
                WriteTerminal(paneID, data);
            });
        });
    });

    // Terminal Toolbar Button Handlers
    document.getElementById("btn-close-term")?.addEventListener("click", () => {
        document.getElementById("terminal-modal")?.classList.add("hidden");
        CloseTerminal("pane-1");
        if (sessionTimerInterval) clearInterval(sessionTimerInterval);
    });

    // SFTP Panel Toggle
    const sftpPanel = document.getElementById("sftp-panel")!;
    document.getElementById("btn-toggle-sftp")?.addEventListener("click", () => {
        sftpPanel.classList.toggle("hidden");
    });
    document.getElementById("btn-close-sftp")?.addEventListener("click", () => {
        sftpPanel.classList.add("hidden");
    });

    // Split V / H Handlers
    const containerPanes = document.getElementById("panes-container")!;
    document.getElementById("btn-split-v")?.addEventListener("click", () => {
        containerPanes.className = "flex-1 grid grid-cols-2 gap-2 p-2 bg-[#090d16]";
        if (!document.getElementById("pane-2")) {
            const p2 = document.createElement("div");
            p2.id = "pane-2";
            p2.className = "w-full h-full border border-[#30363d] rounded p-2 overflow-hidden bg-[#0d1117] flex flex-col";
            p2.innerHTML = `
                <div class="text-[10px] text-[#8b949e] font-mono pb-1 border-b border-[#30363d] flex justify-between items-center">
                    <span>Pane 2 - Secondary SSH Session</span>
                    <button class="hover:text-rose-400" onclick="this.parentElement.parentElement.remove()">✕ Close</button>
                </div>
                <div class="flex-1 w-full h-full" id="term-screen-2"></div>
            `;
            containerPanes.appendChild(p2);
            
            const term2 = new Terminal({ cursorBlink: true, theme: { background: '#0d1117', foreground: '#e6edf3' } });
            const fit2 = new FitAddon();
            term2.loadAddon(fit2);
            term2.open(document.getElementById("term-screen-2")!);
            fit2.fit();
            term2.writeln('\x1b[32m[Secondary Terminal Pane Split Ready]\x1b[0m\r\n$ ');
        }
    });

    document.getElementById("btn-split-h")?.addEventListener("click", () => {
        containerPanes.className = "flex-1 grid grid-rows-2 gap-2 p-2 bg-[#090d16]";
    });

    // EDIT handler
    document.querySelectorAll('.btn-edit-host').forEach(btn => {
        btn.addEventListener('click', (e) => {
            e.stopPropagation();
            const id = (btn as HTMLElement).getAttribute('data-edit-host');
            const h = hosts.find(item => item.id === id);
            if (!h) return;

            editingHostId = h.id;
            panelTitle.textContent = "Edit host";

            (document.getElementById("host-name") as HTMLInputElement).value = h.name || "";
            (document.getElementById("host-ip") as HTMLInputElement).value = h.hostname || "";
            (document.getElementById("host-port") as HTMLInputElement).value = h.port || 22;
            (document.getElementById("host-user") as HTMLInputElement).value = h.username || "root";
            (document.getElementById("host-auth") as HTMLSelectElement).value = h.auth_type || "password";
            (document.getElementById("host-group") as HTMLSelectElement).value = h.group_id || "";

            panel.classList.remove("translate-x-full");
        });
    });

    // CLONE handler
    document.querySelectorAll('.btn-clone-host').forEach(btn => {
        btn.addEventListener('click', (e) => {
            e.stopPropagation();
            const id = (btn as HTMLElement).getAttribute('data-clone-host');
            const h = hosts.find(item => item.id === id);
            if (!h) return;

            editingHostId = null;
            panelTitle.textContent = "Clone host";

            (document.getElementById("host-name") as HTMLInputElement).value = (h.name || h.hostname) + " (Copy)";
            (document.getElementById("host-ip") as HTMLInputElement).value = h.hostname || "";
            (document.getElementById("host-port") as HTMLInputElement).value = h.port || 22;
            (document.getElementById("host-user") as HTMLInputElement).value = h.username || "root";
            (document.getElementById("host-auth") as HTMLSelectElement).value = h.auth_type || "password";
            (document.getElementById("host-group") as HTMLSelectElement).value = h.group_id || "";

            panel.classList.remove("translate-x-full");
        });
    });

    // DELETE handler
    document.querySelectorAll('.btn-del-host').forEach(btn => {
        btn.addEventListener('click', async (e) => {
            e.stopPropagation();
            const id = (btn as HTMLElement).getAttribute('data-del-host');
            if (id && confirm("Delete host connection?")) {
                await DeleteHost(id);
                renderDashboard(container);
            }
        });
    });

    // SAVE handler
    document.getElementById("btn-save-host")?.addEventListener("click", async () => {
        const form = document.getElementById("add-host-form") as HTMLFormElement;
        if (!form.checkValidity()) {
            form.reportValidity();
            return;
        }

        const input = {
            name: (document.getElementById("host-name") as HTMLInputElement).value,
            hostname: (document.getElementById("host-ip") as HTMLInputElement).value,
            port: parseInt((document.getElementById("host-port") as HTMLInputElement).value, 10),
            username: (document.getElementById("host-user") as HTMLInputElement).value,
            auth_type: (document.getElementById("host-auth") as HTMLSelectElement).value,
            password: (document.getElementById("host-pass") as HTMLInputElement).value,
            private_key: (document.getElementById("host-key") as HTMLTextAreaElement).value,
            group_id: (document.getElementById("host-group") as HTMLSelectElement).value || undefined,
        };

        try {
            if (editingHostId) {
                await UpdateHost(editingHostId, input as any);
            } else {
                await CreateHost(input as any);
            }
            panel.classList.add("translate-x-full");
            renderDashboard(container);
        } catch (e: any) {
            alert("Failed to save host: " + e.toString());
        }
    });
}

function renderGroupsView(mainView: HTMLElement, container: HTMLElement, groups: any[]) {
    mainView.innerHTML = `
        <div class="px-8 py-6 border-b border-[#30363d] flex justify-between items-center bg-[#0d1117]">
            <div>
                <h2 class="text-xl font-bold text-white">Groups</h2>
                <p class="text-sm text-[#8b949e]">Organize host connections into groups or projects</p>
            </div>
            <button id="btn-add-group" class="bg-[#238636] hover:bg-[#2ea043] text-white font-medium py-2 px-4 rounded-md text-sm flex items-center shadow-md transition-colors">
                <svg class="h-4 w-4 mr-1.5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/></svg>
                Add Group
            </button>
        </div>

        <div class="flex-1 overflow-y-auto p-8">
            ${groups.length === 0 ? `
                <div class="h-full flex flex-col items-center justify-center text-center">
                    <svg class="h-12 w-12 text-[#30363d] mb-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 002-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"/></svg>
                    <h3 class="text-lg font-medium text-[#e6edf3]">No groups created</h3>
                    <p class="mt-1 text-sm text-[#8b949e]">Create groups to categorize your host infrastructure.</p>
                </div>
            ` : `
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                    ${groups.map(g => `
                        <div class="bg-[#161b22] border border-[#30363d] rounded-md p-4 flex justify-between items-center">
                            <div>
                                <h3 class="font-medium text-[#58a6ff]">${g.name}</h3>
                                <p class="text-xs text-[#8b949e]">Sort order: ${g.sort_order || 0}</p>
                            </div>
                            <button data-del-group="${g.id}" class="btn-del-group text-xs text-rose-400 hover:text-rose-300">Delete</button>
                        </div>
                    `).join('')}
                </div>
            `}
        </div>
    `;

    document.getElementById("btn-add-group")?.addEventListener("click", async () => {
        const name = prompt("Group Name:");
        if (name && name.trim()) {
            try {
                await CreateGroup(name.trim(), 0);
                renderDashboard(container);
            } catch (e: any) {
                alert("Failed to create group: " + e.toString());
            }
        }
    });

    document.querySelectorAll('.btn-del-group').forEach(btn => {
        btn.addEventListener('click', async () => {
            const id = (btn as HTMLElement).getAttribute('data-del-group');
            if (id && confirm("Delete this group?")) {
                await DeleteGroup(id);
                renderDashboard(container);
            }
        });
    });
}

function renderGenericView(mainView: HTMLElement, tabId: string) {
    const titles: Record<string, string> = {
        snippets: "Snippets",
        teams: "Teams",
        port_forwarding: "Port Forwarding",
        monitoring: "Server Monitoring",
        command_logs: "Command Logs",
        ssh_keys: "SSH Keys Manager"
    };

    mainView.innerHTML = `
        <div class="px-8 py-6 border-b border-[#30363d] bg-[#0d1117]">
            <h2 class="text-xl font-bold text-white">${titles[tabId] || tabId}</h2>
            <p class="text-sm text-[#8b949e]">Module configuration and management</p>
        </div>
        <div class="flex-1 flex flex-col items-center justify-center p-8 text-center">
            <div class="h-16 w-16 rounded-full bg-[#161b22] border border-[#30363d] flex items-center justify-center text-[#38bdf8] mb-4 shadow-lg">&gt;_</div>
            <h3 class="text-lg font-medium text-[#e6edf3] mb-1">${titles[tabId] || tabId}</h3>
            <p class="text-sm text-[#8b949e] max-w-md mb-6">This feature module is actively managed by your local CATerm background tasks.</p>
            <span class="inline-flex items-center px-3 py-1 rounded-full text-xs font-medium bg-[#1f2328] text-emerald-400 border border-emerald-500/30">
                ● Active Module
            </span>
        </div>
    `;
}
