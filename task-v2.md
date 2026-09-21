# CATerm v2 (Tauri + Rust + SvelteKit) Task Tracker

## 1. Project Setup
- [x] Initialize Tauri project with Rust backend and SvelteKit frontend (if not already done).
- [x] Configure `tauri.conf.json` for proper window constraints and permissions.
- [x] Set up TailwindCSS or equivalent for styling in SvelteKit.

## 2. Layout & Global Navigation
- [x] Implement Sidebar/Top bar for Global navigation (Dashboard, Hosts, Groups, Snippets, Settings).
- [x] Implement Sessions Tabs system (e.g., Ctrl+1..3 to switch active sessions).

## 3. Dashboard Welcome Screen
- [x] Design Welcome Page featuring E2EE, SFTP, Audit Log, AI Terminal, and Dedikasi.
- [x] Add "Report Issue" button opening a modal with a mailto link or form to `cecep.azhtech@gmail.com`.

## 4. Hosts Management
- [x] Create UI for listing Hosts.
- [x] Implement "Add Host" modal (Fields: Label, IP, Port, Username, Password/Key, Tags).
- [x] Implement Rust backend commands to store/retrieve Host configurations (Local-First SQLite/JSON).

## 5. Terminal Split Pane (Core UX)
- [x] Integrate xterm.js within a Svelte component.
- [x] Implement split grid system (up to 3-4 terminal instances active).
- [x] Display connection details (Path/IP) on active terminal header.
- [x] Bind Rust SSH client (e.g., `russh` or `ssh2`) to xterm.js frontend.

## 6. Groups Management
- [x] Create UI for Groups.
- [x] Implement "Create Group" modal (Fields: Name, Color Swatch, Multiselect Hosts).
- [x] Link backend logic to map Hosts to Groups.

## 7. Snippets System
- [x] Create UI for Snippets.
- [x] Implement 2-step creation modal:
      - Step 1: Bash Command entry.
      - Step 2: Label, Description, Tags entry.
- [x] Add ability to execute snippet on active terminal session.

## 8. Settings
- [x] Implement Settings view.
- [x] Add 'Update' section.
- [x] Add 'Cloud Sync E2EE' section (Placeholder $1/mo and team $1 marked as "Next Feature/Disabled", not a standard Pro upgrade).

## 9. Security & DB (Zero-Knowledge)
- [x] Ensure SQLite database uses strong encryption (e.g., `sqlcipher` or similar Rust crates).
- [x] Implement minimum 8-character password requirement for vault unlock.
## 10. Audit & Completion
- [x] Audited and completed Hosts, Groups, Snippets, and SSH Keys persistence to SQLite. Updated Hosts toolbar to match screenshot requirements.
