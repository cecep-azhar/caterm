import './style.css';
import { IsInitialized, Setup, Unlock, CheckSyncPending, ApplySync, OpenDonationLink, HardDeleteRecord } from "../wailsjs/go/main/App";

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
        app.innerHTML = `<div class="min-h-screen flex items-center justify-center bg-[#0b0f19]"><div class="text-red-400 p-6 glass-card rounded-xl">Error: ${e}</div></div>`;
    }
});

function renderSetup(container: HTMLElement) {
    container.innerHTML = `
        <div class="min-h-screen flex items-center justify-center bg-[#0b0f19] text-[#f8fafc] font-sans">
            <div class="glass-card p-10 rounded-2xl w-[420px]">
                <div class="flex items-center justify-center mb-6">
                    <div class="h-12 w-12 rounded-full bg-gradient-to-tr from-sky-400 to-blue-600 flex items-center justify-center shadow-[0_0_15px_rgba(56,189,248,0.5)]">
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                        </svg>
                    </div>
                </div>
                <h1 class="text-3xl font-bold mb-2 text-center tracking-tight">Setup Vault</h1>
                <p class="mb-8 text-[#94a3b8] text-center text-sm leading-relaxed">Create a secure master password to encrypt your local database.</p>
                <form id="setup-form" class="space-y-5">
                    <div>
                        <label class="block text-xs font-semibold text-[#94a3b8] uppercase tracking-wider mb-2">Master Password</label>
                        <input type="password" id="password" placeholder="Min 12 characters" class="w-full p-3 rounded-lg custom-input" required>
                        <div class="h-1.5 mt-3 w-full bg-[#1e293b] rounded-full overflow-hidden">
                            <div id="password-strength" class="h-full bg-slate-600 transition-all duration-300 w-0"></div>
                        </div>
                        <p id="password-error" class="text-red-400 text-xs hidden mt-2 font-medium"></p>
                    </div>
                    <div>
                        <label class="block text-xs font-semibold text-[#94a3b8] uppercase tracking-wider mb-2">Confirm Password</label>
                        <input type="password" id="confirm-password" placeholder="Type again to verify" class="w-full p-3 rounded-lg custom-input" required>
                    </div>
                    <div class="flex items-start bg-[#151d2a] p-4 rounded-lg border border-[#334155] mt-6">
                        <input type="checkbox" id="acknowledge" class="mt-1 mr-3 w-4 h-4 rounded border-[#334155] text-sky-500 bg-[#1e293b] focus:ring-sky-500" required>
                        <label for="acknowledge" class="text-xs text-[#94a3b8] leading-tight">
                            I understand there is <strong class="text-red-400 font-semibold">NO recovery option</strong> if I lose this master password.
                        </label>
                    </div>
                    <button type="submit" id="submit-btn" class="w-full glow-btn text-white font-bold py-3 px-4 rounded-lg mt-6" disabled>Initialize Secure Vault</button>
                </form>
            </div>
        </div>
    `;

    const form = document.getElementById("setup-form") as HTMLFormElement;
    const pwdInput = document.getElementById("password") as HTMLInputElement;
    const confirmInput = document.getElementById("confirm-password") as HTMLInputElement;
    const ackInput = document.getElementById("acknowledge") as HTMLInputElement;
    const submitBtn = document.getElementById("submit-btn") as HTMLButtonElement;
    const strengthIndicator = document.getElementById("password-strength") as HTMLDivElement;
    const errorText = document.getElementById("password-error") as HTMLParagraphElement;

    const validateForm = () => {
        const pwd = pwdInput.value;
        let valid = true;
        let strength = 0;
        
        if (pwd.length >= 12) {
             strength = 33;
             if (/[A-Z]/.test(pwd) && /[a-z]/.test(pwd)) strength += 33;
             if (/[0-9]/.test(pwd) || /[^A-Za-z0-9]/.test(pwd)) strength += 34;
        } else if (pwd.length > 0) {
             strength = (pwd.length / 12) * 33;
        }

        strengthIndicator.style.width = `${strength}%`;
        if (strength < 33 && pwd.length > 0) {
            strengthIndicator.className = "h-full transition-all duration-300 bg-red-500";
            errorText.textContent = "Password must be at least 12 characters.";
            errorText.classList.remove("hidden");
            valid = false;
        } else if (strength < 66 && pwd.length >= 12) {
             strengthIndicator.className = "h-full transition-all duration-300 bg-amber-500";
             errorText.classList.add("hidden");
        } else if (strength >= 66) {
             strengthIndicator.className = "h-full transition-all duration-300 bg-emerald-500 shadow-[0_0_8px_#10b981]";
             errorText.classList.add("hidden");
        } else {
             strengthIndicator.className = "h-full transition-all duration-300 bg-slate-600";
             errorText.classList.add("hidden");
             valid = false;
        }

        if (pwd !== confirmInput.value && confirmInput.value !== "") {
            valid = false;
        }
        if (!ackInput.checked) valid = false;

        submitBtn.disabled = !valid;
    };

    pwdInput.addEventListener("input", validateForm);
    confirmInput.addEventListener("input", validateForm);
    ackInput.addEventListener("change", validateForm);

    form.addEventListener("submit", async (e) => {
        e.preventDefault();
        try {
            await Setup(pwdInput.value);
            renderUnlock(container);
        } catch (err: any) {
             errorText.textContent = err.toString();
             errorText.classList.remove("hidden");
        }
    });
}

function renderUnlock(container: HTMLElement) {
    container.innerHTML = `
        <div id="unlock-container" class="min-h-screen flex items-center justify-center bg-[#0b0f19] text-[#f8fafc] transition-transform duration-100 font-sans">
            <div class="glass-card p-10 rounded-2xl w-[400px]">
                <div class="flex flex-col items-center justify-center mb-8">
                    <div class="h-16 w-16 rounded-full bg-[#1e293b] border border-[#334155] flex items-center justify-center mb-4 shadow-[0_0_20px_rgba(0,0,0,0.5)]">
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-8 w-8 text-sky-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                        </svg>
                    </div>
                    <h1 class="text-3xl font-bold tracking-tight">Unlock Vault</h1>
                    <p class="text-[#94a3b8] text-sm mt-2">Enter your master password to continue</p>
                </div>
                <form id="unlock-form" class="space-y-6">
                    <div>
                        <input type="password" id="unlock-password" placeholder="Master Password" class="w-full p-4 rounded-lg custom-input text-lg tracking-wider" required>
                        <p id="unlock-error" class="text-red-400 text-xs hidden mt-2 font-medium"></p>
                    </div>
                    <button type="submit" class="w-full glow-btn text-white font-bold py-4 px-4 rounded-lg text-lg uppercase tracking-wider shadow-lg">Unlock</button>
                </form>
            </div>
        </div>
    `;

    const form = document.getElementById("unlock-form") as HTMLFormElement;
    const pwdInput = document.getElementById("unlock-password") as HTMLInputElement;
    const errorText = document.getElementById("unlock-error") as HTMLParagraphElement;
    const unlockContainer = document.getElementById("unlock-container") as HTMLDivElement;

    // Focus immediately
    setTimeout(() => pwdInput.focus(), 100);

    form.addEventListener("submit", async (e) => {
        e.preventDefault();
        
        // Add loading state
        const submitBtn = form.querySelector('button[type="submit"]') as HTMLButtonElement;
        const originalText = submitBtn.textContent;
        submitBtn.innerHTML = `<svg class="animate-spin h-5 w-5 mx-auto text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24"><circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle><path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path></svg>`;
        submitBtn.disabled = true;

        try {
            await Unlock(pwdInput.value);
            renderDashboard(container);
        } catch (err: any) {
             errorText.textContent = "Incorrect master password. Please try again.";
             errorText.classList.remove("hidden");
             pwdInput.classList.add("border-red-500", "focus:border-red-500", "focus:ring-red-500/20");
             
             // Restore button
             submitBtn.innerHTML = originalText || "Unlock";
             submitBtn.disabled = false;
             
             // Shake effect
             unlockContainer.classList.add("translate-x-2");
             setTimeout(() => unlockContainer.classList.replace("translate-x-2", "-translate-x-2"), 50);
             setTimeout(() => unlockContainer.classList.replace("-translate-x-2", "translate-x-2"), 100);
             setTimeout(() => unlockContainer.classList.replace("translate-x-2", "-translate-x-2"), 150);
             setTimeout(() => unlockContainer.classList.remove("-translate-x-2"), 200);
             
             pwdInput.value = "";
             pwdInput.focus();
        }
    });
    
    // Clear error style on input
    pwdInput.addEventListener('input', () => {
        pwdInput.classList.remove("border-red-500", "focus:border-red-500", "focus:ring-red-500/20");
        errorText.classList.add("hidden");
    });
}

function renderSyncReview(container: HTMLElement, syncResult: any) {
    container.innerHTML = `
        <div class="min-h-screen flex items-center justify-center bg-[#0b0f19] text-[#f8fafc]">
            <div class="glass-card p-8 rounded-2xl w-[450px]">
                <h2 class="text-2xl font-bold mb-6 text-sky-400">Sync Pending</h2>
                <div class="space-y-4 mb-8 bg-[#151d2a] p-4 rounded-xl border border-[#1e293b]">
                    <div class="flex justify-between items-center pb-2 border-b border-[#1e293b]">
                        <span class="text-[#94a3b8]">Applied changes:</span>
                        <span class="font-bold text-emerald-400 px-2 py-1 bg-emerald-400/10 rounded">${syncResult.applied || 0}</span>
                    </div>
                    <div class="flex justify-between items-center pb-2 border-b border-[#1e293b]">
                        <span class="text-[#94a3b8]">Deleted records:</span>
                        <span class="font-bold text-rose-400 px-2 py-1 bg-rose-400/10 rounded">${syncResult.deleted || 0}</span>
                    </div>
                    <div class="flex justify-between items-center">
                        <span class="text-[#94a3b8]">Conflicts:</span>
                        <span class="font-bold text-amber-400 px-2 py-1 bg-amber-400/10 rounded">${syncResult.conflicts || 0}</span>
                    </div>
                </div>
                <div class="flex space-x-4">
                    <button id="sync-cancel" class="flex-1 bg-[#1e293b] hover:bg-[#334155] border border-[#334155] text-white font-semibold py-3 px-4 rounded-lg transition-colors">Skip for now</button>
                    <button id="sync-apply" class="flex-1 glow-btn text-white font-bold py-3 px-4 rounded-lg shadow-lg">Apply Sync</button>
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
            alert("Gagal mengaplikasikan sync: " + e.toString());
        }
    });
}

function renderDashboard(container: HTMLElement) {
    container.innerHTML = `
        <div class="min-h-screen bg-[#0b0f19] text-[#f8fafc] flex flex-col font-sans">
            <!-- Topbar -->
            <div class="px-6 py-4 border-b border-[#1e293b] bg-[rgba(21,29,42,0.6)] backdrop-blur-md flex justify-between items-center sticky top-0 z-10">
                <div class="flex items-center space-x-3">
                    <div class="h-8 w-8 rounded-lg bg-gradient-to-tr from-sky-400 to-blue-600 flex items-center justify-center">
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                        </svg>
                    </div>
                    <h1 class="text-xl font-bold tracking-wide">CATERM <span class="text-sky-500 font-medium text-sm ml-2">v1.0.0</span></h1>
                </div>
                <button id="btn-settings" class="h-10 w-10 rounded-full bg-[#1e293b] border border-[#334155] flex items-center justify-center text-[#94a3b8] hover:text-white hover:bg-[#334155] transition-all">
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                    </svg>
                </button>
            </div>
            
            <!-- Main Content Area -->
            <div class="p-8 flex-1 flex flex-col items-center justify-center">
                <div class="text-center">
                    <div class="inline-flex items-center justify-center h-24 w-24 rounded-full bg-emerald-500/10 mb-6 border border-emerald-500/20">
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-12 w-12 text-emerald-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
                        </svg>
                    </div>
                    <h2 class="text-3xl font-bold mb-2">Vault Unlocked Successfully</h2>
                    <p class="text-[#94a3b8] max-w-md mx-auto">Your database is decrypted and ready. Connections, groups, and synced configurations are fully accessible.</p>
                </div>
            </div>
        </div>
    `;

    document.getElementById("btn-settings")?.addEventListener("click", () => {
        renderSettings(container);
    });
}

function renderSettings(container: HTMLElement) {
    container.innerHTML = `
        <div class="min-h-screen bg-[#0b0f19] text-[#f8fafc] flex flex-col font-sans">
            <!-- Topbar -->
            <div class="px-6 py-4 border-b border-[#1e293b] bg-[rgba(21,29,42,0.6)] backdrop-blur-md flex items-center sticky top-0 z-10">
                <button id="btn-back" class="h-10 w-10 rounded-full bg-[#1e293b] border border-[#334155] flex items-center justify-center text-[#94a3b8] hover:text-white hover:bg-[#334155] transition-all mr-4">
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18" />
                    </svg>
                </button>
                <h1 class="text-xl font-bold">Settings</h1>
            </div>
            
            <div class="p-8 max-w-3xl mx-auto w-full">
                <!-- Danger Zone Section -->
                <section class="glass-card p-8 rounded-2xl border border-rose-500/30 shadow-[0_0_20px_rgba(244,63,94,0.05)] relative overflow-hidden">
                    <div class="absolute top-0 left-0 w-full h-1 bg-gradient-to-r from-rose-500 to-red-600"></div>
                    <div class="flex items-center mb-6">
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6 text-rose-500 mr-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                        </svg>
                        <h2 class="text-xl font-bold text-rose-400">Danger Zone</h2>
                    </div>
                    
                    <div class="space-y-5">
                        <div>
                            <h3 class="text-[#f8fafc] font-semibold mb-1">Permanent Record Deletion</h3>
                            <p class="text-sm text-[#94a3b8]">Hard delete a record permanently from the database. This action bypasses soft delete and cannot be undone.</p>
                        </div>
                        
                        <div class="flex flex-col sm:flex-row space-y-3 sm:space-y-0 sm:space-x-3 bg-[#0b0f19] p-4 rounded-xl border border-[#1e293b]">
                            <select id="del-table" class="bg-[#1e293b] border border-[#334155] rounded-lg p-3 text-white focus:outline-none focus:border-rose-500 focus:ring-1 focus:ring-rose-500">
                                <option value="groups">Groups</option>
                                <option value="hosts">Hosts</option>
                            </select>
                            <input type="text" id="del-id" placeholder="Record ID (UUIDv7)" class="flex-1 bg-[#1e293b] border border-[#334155] rounded-lg p-3 text-white focus:outline-none focus:border-rose-500 focus:ring-1 focus:ring-rose-500 placeholder-slate-500">
                            <button id="btn-hard-delete" class="bg-gradient-to-r from-rose-600 to-red-600 hover:from-rose-500 hover:to-red-500 text-white px-6 py-3 rounded-lg font-bold shadow-[0_4px_14px_0_rgba(225,29,72,0.39)] transition-all">Destroy</button>
                        </div>
                    </div>
                </section>
            </div>
        </div>
    `;

    document.getElementById("btn-back")?.addEventListener("click", () => {
        renderDashboard(container);
    });

    document.getElementById("btn-hard-delete")?.addEventListener("click", async () => {
        const table = (document.getElementById("del-table") as HTMLSelectElement).value;
        const id = (document.getElementById("del-id") as HTMLInputElement).value;
        if (!id) return;
        
        if (confirm(`⚠️ WARNING: Are you absolutely sure you want to permanently delete record ${id} from ${table}? This cannot be undone.`)) {
            try {
                await HardDeleteRecord(table, id);
                alert("Record deleted successfully.");
                (document.getElementById("del-id") as HTMLInputElement).value = "";
            } catch(e: any) {
                alert("Failed to delete record: " + e.toString());
            }
        }
    });
}
