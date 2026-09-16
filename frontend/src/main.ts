import { IsInitialized, Setup, Unlock } from "../wailsjs/go/main/App";

document.addEventListener("DOMContentLoaded", async () => {
    const app = document.getElementById("app");
    if (!app) return;

    try {
        const isInit = await IsInitialized();
        if (isInit) {
            renderUnlock(app);
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
        <div class="min-h-screen bg-gray-900 text-white p-4">
            <h1 class="text-2xl font-bold">Dashboard</h1>
            <p>Vault unlocked successfully.</p>
        </div>
    `;
}
