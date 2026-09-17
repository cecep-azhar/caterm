import './style.css';
import { IsInitialized, Setup, Unlock, CheckSyncPending, ApplySync, OpenDonationLink, HardDeleteRecord, ListHosts, CreateHost, ListGroups, ResetVault } from "../wailsjs/go/main/App";

document.addEventListener("DOMContentLoaded", async () => {
    const app = document.getElementById("app");
    if (!app) return;

    try {
        const isInit = await IsInitialized();
        if (isInit) {
            const syncResult = await CheckSyncPending();
            if (syncResult && (syncResult.applied > 0 || syncResult.deleted > 0 || syncResult.conflicts > 0)) {
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

function renderSetup(container: HTMLElement) {
    container.innerHTML = `
        <div class="min-h-screen flex bg-[#0d1117] text-[#e6edf3] font-sans">
            <!-- Left Column -->
            <div class="flex-1 flex flex-col justify-between p-12 bg-[#010409] border-r border-[#30363d]">
                <div>
                    <div class="flex items-center space-x-3 mb-2">
                        <div class="h-10 w-10 rounded-xl bg-[#0b0f19] flex items-center justify-center font-mono font-bold text-[#38bdf8] shadow-lg text-lg tracking-wider border border-[#38bdf8]/30">
                            &gt;_
                        </div>
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

            <!-- Right Column -->
            <div class="w-[480px] flex flex-col justify-center p-12 bg-[#0d1117]">
                <div class="mb-8">
                    <h2 class="text-3xl font-bold mb-2">Setup Vault</h2>
                    <p class="text-[#8b949e]">cecep.azhtech@gmail.com</p>
                </div>
                
                <form id="setup-form" class="space-y-5">
                    <div>
                        <label class="block text-sm font-medium mb-2 text-[#e6edf3]">Master Password (min 12 chars)</label>
                        <input type="password" id="setup-password" class="w-full p-3 rounded-md custom-input" placeholder="Enter master password" required>
                        <p id="setup-error" class="text-[#f85149] text-sm hidden mt-2"></p>
                    </div>
                    <div>
                        <label class="block text-sm font-medium mb-2 text-[#e6edf3]">Confirm Password</label>
                        <input type="password" id="setup-confirm" class="w-full p-3 rounded-md custom-input" placeholder="Confirm master password" required>
                    </div>
                    <button type="submit" class="w-full bg-[#238636] hover:bg-[#2ea043] text-white font-semibold py-3 px-4 rounded-md transition-colors shadow-lg">Initialize Vault</button>
                </form>
            </div>
        </div>
    `;

    const form = document.getElementById("setup-form") as HTMLFormElement;
    const pwdInput = document.getElementById("setup-password") as HTMLInputElement;
    const confirmInput = document.getElementById("setup-confirm") as HTMLInputElement;
    const errorText = document.getElementById("setup-error") as HTMLParagraphElement;

    form.addEventListener("submit", async (e) => {
        e.preventDefault();
        const pwd = pwdInput.value;
        if (!pwd || pwd.length < 12) {
            errorText.textContent = "Master Password must be at least 12 characters.";
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
            <!-- Left Column -->
            <div class="flex-1 flex flex-col justify-between p-12 bg-[#010409] border-r border-[#30363d]">
                <div>
                    <div class="flex items-center space-x-3 mb-2">
                        <div class="h-10 w-10 rounded-xl bg-[#0b0f19] flex items-center justify-center font-mono font-bold text-[#38bdf8] shadow-lg text-lg tracking-wider border border-[#38bdf8]/30">
                            &gt;_
                        </div>
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

            <!-- Right Column -->
            <div class="w-[480px] flex flex-col justify-center p-12 bg-[#0d1117]">
                <div class="mb-8">
                    <h2 class="text-3xl font-bold mb-2">Welcome back</h2>
                    <p class="text-[#8b949e]">cecep.azhtech@gmail.com</p>
                </div>
                
                <form id="unlock-form" class="space-y-6" autocomplete="off">
                    <div>
                        <label class="block text-sm font-medium mb-2 text-[#e6edf3]">Master Password</label>
                        <input type="password" id="unlock-password" class="w-full p-3 rounded-md custom-input" placeholder="Enter master password" required autofocus autocomplete="current-password">
                        <p id="unlock-error" class="text-[#f85149] text-sm hidden mt-2"></p>
                    </div>
                    <button type="submit" class="w-full bg-[#238636] hover:bg-[#2ea043] text-white font-semibold py-3 px-4 rounded-md transition-colors shadow-lg">Unlock</button>
                    <button type="button" id="btn-reset-vault" class="w-full mt-2 bg-transparent hover:bg-rose-500/10 text-rose-500 border border-rose-500/20 font-semibold py-2 px-4 rounded-md transition-colors text-sm">Reset Vault</button>
                </form>
            </div>
        </div>
    `;

    const form = document.getElementById("unlock-form") as HTMLFormElement;
    const pwdInput = document.getElementById("unlock-password") as HTMLInputElement;
    const errorText = document.getElementById("unlock-error") as HTMLParagraphElement;

    // Ensure error message is explicitly hidden on initial render
    errorText.classList.add("hidden");
    errorText.textContent = "";

    setTimeout(() => pwdInput.focus(), 100);

    form.addEventListener("submit", async (e) => {
        e.preventDefault();
        
        const pwd = pwdInput.value;
        if (!pwd || pwd.trim() === "") {
            // Do not submit empty string or show incorrect password
            return;
        }
        
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
        if (confirm("Reset Vault will permanently delete your local database. All saved hosts and settings will be lost. Continue?")) {
            try {
                await ResetVault();
                location.reload();
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
                        <span class="text-[#8b949e]">Applied changes:</span>
                        <span class="font-bold text-[#3fb950]">${syncResult.applied || 0}</span>
                    </div>
                    <div class="flex justify-between items-center pb-2 border-b border-[#30363d]">
                        <span class="text-[#8b949e]">Deleted records:</span>
                        <span class="font-bold text-[#f85149]">${syncResult.deleted || 0}</span>
                    </div>
                    <div class="flex justify-between items-center">
                        <span class="text-[#8b949e]">Conflicts:</span>
                        <span class="font-bold text-[#d29922]">${syncResult.conflicts || 0}</span>
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

    container.innerHTML = `
        <div class="h-screen flex bg-[#0d1117] text-[#e6edf3] font-sans overflow-hidden">
            <!-- Sidebar -->
            <div class="w-64 flex flex-col bg-[#010409] border-r border-[#30363d]">
                <div class="p-4 border-b border-[#30363d] flex items-center space-x-3">
                    <div class="h-8 w-8 rounded-lg bg-[#0b0f19] flex items-center justify-center font-mono font-bold text-[#38bdf8] text-xs shadow-md border border-[#38bdf8]/30">
                        &gt;_
                    </div>
                    <h1 class="text-xl font-bold tracking-tight text-white">CATerm</h1>
                </div>
                
                <div class="flex-1 overflow-y-auto py-4">
                    <nav class="space-y-1 px-2">
                        <a href="#" class="flex items-center px-3 py-2 bg-[#161b22] text-[#e6edf3] rounded-md font-medium text-sm group border border-[#30363d]">
                            <span class="mr-3 text-[#e6edf3]">
                                <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M12 5l7 7-7 7"/></svg>
                            </span>
                            Hosts
                        </a>
                        <a href="#" class="flex items-center px-3 py-2 text-[#8b949e] hover:bg-[#161b22] hover:text-[#e6edf3] rounded-md font-medium text-sm group transition-colors">
                            <span class="mr-3 text-[#8b949e] group-hover:text-[#e6edf3]">
                                <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 002-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"/></svg>
                            </span>
                            Groups
                        </a>
                        <a href="#" class="flex items-center px-3 py-2 text-[#8b949e] hover:bg-[#161b22] hover:text-[#e6edf3] rounded-md font-medium text-sm group transition-colors">
                            <span class="mr-3 text-[#8b949e] group-hover:text-[#e6edf3]">
                                <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/></svg>
                            </span>
                            Snippets
                        </a>
                        <a href="#" class="flex items-center px-3 py-2 text-[#8b949e] hover:bg-[#161b22] hover:text-[#e6edf3] rounded-md font-medium text-sm group transition-colors">
                            <span class="mr-3 text-[#8b949e] group-hover:text-[#e6edf3]">
                                <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"/></svg>
                            </span>
                            Teams
                        </a>
                        <a href="#" class="flex items-center px-3 py-2 text-[#8b949e] hover:bg-[#161b22] hover:text-[#e6edf3] rounded-md font-medium text-sm group transition-colors">
                            <span class="mr-3 text-[#8b949e] group-hover:text-[#e6edf3]">
                                <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4"/></svg>
                            </span>
                            Port forwarding
                        </a>
                        <a href="#" class="flex items-center px-3 py-2 text-[#8b949e] hover:bg-[#161b22] hover:text-[#e6edf3] rounded-md font-medium text-sm group transition-colors">
                            <span class="mr-3 text-[#8b949e] group-hover:text-[#e6edf3]">
                                <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"/></svg>
                            </span>
                            Monitoring
                        </a>
                        <a href="#" class="flex items-center px-3 py-2 text-[#8b949e] hover:bg-[#161b22] hover:text-[#e6edf3] rounded-md font-medium text-sm group transition-colors">
                            <span class="mr-3 text-[#8b949e] group-hover:text-[#e6edf3]">
                                <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"/></svg>
                            </span>
                            Command logs
                        </a>
                        <a href="#" class="flex items-center px-3 py-2 text-[#8b949e] hover:bg-[#161b22] hover:text-[#e6edf3] rounded-md font-medium text-sm group transition-colors">
                            <span class="mr-3 text-[#8b949e] group-hover:text-[#e6edf3]">
                                <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z"/></svg>
                            </span>
                            SSH keys
                        </a>
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
                    <svg id="btn-settings" class="h-5 w-5 text-[#8b949e] group-hover:text-[#e6edf3]" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" /><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" /></svg>
                </div>
            </div>

            <!-- Main Content Area -->
            <div class="flex-1 flex flex-col relative overflow-hidden">
                <!-- Topbar -->
                <div class="px-8 py-6 border-b border-[#30363d] flex justify-between items-center bg-[#0d1117]">
                    <div class="flex-1 max-w-xl">
                        <div class="relative">
                            <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                                <svg class="h-5 w-5 text-[#8b949e]" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/></svg>
                            </div>
                            <input type="text" placeholder="Search hosts..." class="block w-full pl-10 pr-3 py-2 border border-[#30363d] rounded-md leading-5 bg-[#0d1117] text-[#e6edf3] placeholder-[#8b949e] focus:outline-none focus:border-[#58a6ff] focus:ring-1 focus:ring-[#58a6ff] sm:text-sm transition-colors">
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

                <!-- Content -->
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
                                <div class="bg-[#161b22] border border-[#30363d] rounded-md p-4 hover:border-[#8b949e] transition-colors cursor-pointer group">
                                    <div class="flex justify-between items-start mb-2">
                                        <h3 class="font-medium text-[#58a6ff] group-hover:underline">${h.label || h.host}</h3>
                                        <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-[#1f2328] text-[#8b949e] border border-[#30363d]">${h.port}</span>
                                    </div>
                                    <p class="text-sm text-[#8b949e] truncate">${h.username}@${h.host}</p>
                                </div>
                            `).join('')}
                        </div>
                    `}
                </div>

                <!-- Slide-out Panel (Add Host) -->
                <div id="slide-panel" class="absolute inset-y-0 right-0 w-96 bg-[#161b22] border-l border-[#30363d] transform translate-x-full transition-transform duration-300 ease-in-out z-20 shadow-2xl flex flex-col">
                    <div class="px-6 py-4 border-b border-[#30363d] flex justify-between items-center bg-[#010409]">
                        <h2 class="text-lg font-medium">Add new host</h2>
                        <button id="btn-close-panel" class="text-[#8b949e] hover:text-[#e6edf3] transition-colors">
                            <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/></svg>
                        </button>
                    </div>
                    
                    <div class="flex-1 overflow-y-auto p-6">
                        <form id="add-host-form" class="space-y-5">
                            <div>
                                <label class="block text-sm font-medium mb-1 text-[#e6edf3]">Label</label>
                                <input type="text" id="host-label" placeholder="e.g. Production Web" class="w-full p-2.5 rounded-md custom-input text-sm">
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
                                <input type="password" id="host-pass" class="w-full p-2.5 rounded-md custom-input text-sm">
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
            </div>
        </div>
    `;

    // Slide Panel Logic
    const panel = document.getElementById("slide-panel")!;
    const btnAdd = document.getElementById("btn-add-host")!;
    const btnClose = document.getElementById("btn-close-panel")!;
    
    btnAdd.addEventListener("click", () => {
        panel.classList.remove("translate-x-full");
    });
    
    btnClose.addEventListener("click", () => {
        panel.classList.add("translate-x-full");
    });

    // Auth method toggle
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

    // Save Host
    document.getElementById("btn-save-host")?.addEventListener("click", async () => {
        const form = document.getElementById("add-host-form") as HTMLFormElement;
        if (!form.checkValidity()) {
            form.reportValidity();
            return;
        }

        const input = {
            label: (document.getElementById("host-label") as HTMLInputElement).value,
            host: (document.getElementById("host-ip") as HTMLInputElement).value,
            port: parseInt((document.getElementById("host-port") as HTMLInputElement).value, 10),
            username: (document.getElementById("host-user") as HTMLInputElement).value,
            auth_method: (document.getElementById("host-auth") as HTMLSelectElement).value,
            password: (document.getElementById("host-pass") as HTMLInputElement).value,
            private_key: (document.getElementById("host-key") as HTMLTextAreaElement).value,
            group_id: (document.getElementById("host-group") as HTMLSelectElement).value || undefined,
            tags: []
        };

        try {
            await CreateHost(input as any);
            panel.classList.add("translate-x-full");
            renderDashboard(container); // reload
        } catch (e: any) {
            alert("Failed to save host: " + e.toString());
        }
    });
}
