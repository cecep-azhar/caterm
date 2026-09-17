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
        app.innerHTML = `<div class="text-red-500">Error: ${e}</div>`;
    }
});

function renderSetup(container: HTMLElement) {
    container.innerHTML = `
        <div class="min-h-screen flex items-center justify-center bg-gray-900 text-white">
            <div class="bg-gray-800 p-8 rounded-lg shadow-lg w-96">
                <h1 class="text-2xl font-bold mb-4">Welcome to CATERM</h1>
                <p class="mb-4 text-gray-300">Let's set up your master password.</p>
                <form id="setup-form" class="space-y-4">
                    <div>
                        <input type="password" id="password" placeholder="Master Password (min 12 chars)" class="w-full p-2 rounded bg-gray-700 text-white border border-gray-600 focus:outline-none focus:border-blue-500" required>
                        <div id="password-strength" class="h-2 mt-1 rounded bg-gray-600 transition-all duration-300"></div>
                        <p id="password-error" class="text-red-400 text-sm hidden"></p>
                    </div>
                    <div>
                        <input type="password" id="confirm-password" placeholder="Confirm Password" class="w-full p-2 rounded bg-gray-700 text-white border border-gray-600 focus:outline-none focus:border-blue-500" required>
                    </div>
                    <div class="flex items-start">
                        <input type="checkbox" id="acknowledge" class="mt-1 mr-2" required>
                        <label for="acknowledge" class="text-sm text-gray-400">I understand there is no recovery option if I lose this password.</label>
                    </div>
                    <button type="submit" id="submit-btn" class="w-full bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded disabled:opacity-50 disabled:cursor-not-allowed" disabled>Initialize Vault</button>
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
        }

        strengthIndicator.style.width = `${strength}%`;
        if (strength < 33) {
            strengthIndicator.className = "h-2 mt-1 rounded transition-all duration-300 bg-red-500";
            errorText.textContent = "Password must be at least 12 characters.";
            errorText.classList.remove("hidden");
            valid = false;
        } else if (strength < 66) {
             strengthIndicator.className = "h-2 mt-1 rounded transition-all duration-300 bg-yellow-500";
             errorText.classList.add("hidden");
        } else {
             strengthIndicator.className = "h-2 mt-1 rounded transition-all duration-300 bg-green-500";
             errorText.classList.add("hidden");
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
        <div id="unlock-container" class="min-h-screen flex items-center justify-center bg-gray-900 text-white transition-transform duration-100">
            <div class="bg-gray-800 p-8 rounded-lg shadow-lg w-96">
                <h1 class="text-2xl font-bold mb-4">Unlock Vault</h1>
                <form id="unlock-form" class="space-y-4">
                    <div>
                        <input type="password" id="unlock-password" placeholder="Master Password" class="w-full p-2 rounded bg-gray-700 text-white border border-gray-600 focus:outline-none focus:border-blue-500" required>
                        <p id="unlock-error" class="text-red-400 text-sm hidden mt-1"></p>
                    </div>
                    <button type="submit" class="w-full bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded">Unlock</button>
                </form>
            </div>
        </div>
    `;

    const form = document.getElementById("unlock-form") as HTMLFormElement;
    const pwdInput = document.getElementById("unlock-password") as HTMLInputElement;
    const errorText = document.getElementById("unlock-error") as HTMLParagraphElement;
    const unlockContainer = document.getElementById("unlock-container") as HTMLDivElement;

    form.addEventListener("submit", async (e) => {
        e.preventDefault();
        try {
            await Unlock(pwdInput.value);
            renderDashboard(container);
        } catch (err: any) {
             errorText.textContent = err.toString();
             errorText.classList.remove("hidden");
             
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
}

function renderDashboard(container: HTMLElement) {
    container.innerHTML = `
        <div class="min-h-screen bg-gray-900 text-white flex flex-col">
            <div class="p-4 border-b border-gray-800 flex justify-between items-center">
                <h1 class="text-2xl font-bold">Dashboard</h1>
                <button id="btn-settings" class="text-gray-400 hover:text-white">
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                    </svg>
                </button>
            </div>
            <div class="p-4 flex-1">
                <p>Vault unlocked successfully.</p>
            </div>
        </div>
    `;

    document.getElementById("btn-settings")?.addEventListener("click", () => {
        renderSettings(container);
    });
}

function renderSettings(container: HTMLElement) {
    container.innerHTML = `
        <div class="min-h-screen bg-gray-900 text-white p-4">
            <div class="flex items-center mb-6">
                <button id="btn-back" class="mr-4 text-gray-400 hover:text-white">
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18" />
                    </svg>
                </button>
                <h1 class="text-2xl font-bold">Settings</h1>
            </div>
            
            <div class="max-w-2xl bg-gray-800 rounded-lg p-6 space-y-8">
                <!-- Delete Data Section -->
                <section>
                    <h2 class="text-xl font-semibold mb-4 text-red-400 border-b border-gray-700 pb-2">Danger Zone</h2>
                    <div class="space-y-4">
                        <p class="text-sm text-gray-400">Hard delete a record permanently from the database (bypasses soft delete).</p>
                        <div class="flex space-x-2">
                            <select id="del-table" class="bg-gray-700 border border-gray-600 rounded p-2 focus:outline-none">
                                <option value="groups">Group</option>
                                <option value="hosts">Host</option>
                            </select>
                            <input type="text" id="del-id" placeholder="Record ID (UUIDv7)" class="flex-1 bg-gray-700 border border-gray-600 rounded p-2 focus:outline-none">
                            <button id="btn-hard-delete" class="bg-red-600 hover:bg-red-700 text-white px-4 py-2 rounded font-medium">Delete</button>
                        </div>
                    </div>
                </section>

                <!-- Support Section -->
                <section>
                    <h2 class="text-xl font-semibold mb-4 border-b border-gray-700 pb-2">Support</h2>
                    <button id="btn-donate" class="flex items-center bg-pink-600 hover:bg-pink-700 text-white px-4 py-2 rounded font-medium">
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 mr-2" viewBox="0 0 20 20" fill="currentColor">
                            <path fill-rule="evenodd" d="M3.172 5.172a4 4 0 015.656 0L10 6.343l1.172-1.171a4 4 0 115.656 5.656L10 17.657l-6.828-6.829a4 4 0 010-5.656z" clip-rule="evenodd" />
                        </svg>
                        Support the Developer (Open in Browser)
                    </button>
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
        
        if (!id) {
            alert("Please enter a valid record ID");
            return;
        }

        if (confirm(`WARNING: This will permanently delete the record with ID ${id} from the ${table} table. This action cannot be undone and will not sync as a deletion to other devices. Continue?`)) {
            try {
                await HardDeleteRecord(table, id);
                alert("Record deleted successfully.");
                (document.getElementById("del-id") as HTMLInputElement).value = "";
            } catch (e: any) {
                alert(`Error deleting record: ${e}`);
            }
        }
    });

    document.getElementById("btn-donate")?.addEventListener("click", () => {
        OpenDonationLink();
    });
}

function renderSyncReview(container: HTMLElement, syncResult: any) {
    container.innerHTML = `
        <div class="min-h-screen flex items-center justify-center bg-gray-900 text-white">
            <div class="bg-gray-800 p-8 rounded-lg shadow-lg w-96">
                <h1 class="text-2xl font-bold mb-4">Pending Sync Changes</h1>
                <p class="mb-4 text-gray-300">New sync data is available from remote:</p>
                <div class="bg-gray-700 p-4 rounded mb-6 space-y-2 text-sm">
                    <div class="flex justify-between">
                        <span>New / Updated:</span>
                        <span class="font-semibold text-green-400">${syncResult.applied || 0}</span>
                    </div>
                    <div class="flex justify-between">
                        <span>Deleted:</span>
                        <span class="font-semibold text-yellow-400">${syncResult.deleted || 0}</span>
                    </div>
                    <div class="flex justify-between">
                        <span>Conflicts:</span>
                        <span class="font-semibold text-red-400">${syncResult.conflicts || 0}</span>
                    </div>
                </div>
                <div class="flex space-x-4">
                    <button id="sync-cancel" class="w-1/2 bg-gray-600 hover:bg-gray-500 text-white font-bold py-2 px-4 rounded">Batal</button>
                    <button id="sync-apply" class="w-1/2 bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded">Terapkan</button>
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
