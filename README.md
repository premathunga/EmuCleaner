# EmuCleaner v0.2 (beta)

Safe cleaner and booster for gamers who play mobile games on Windows emulators.

## What it does
- **Home**: Game Ready Score (free RAM, disk space, CPU load), detected emulators (BlueStacks, LDPlayer, Gameloop), Boost & Play, Quick Clean.
- **Clean**: Windows temp files, crash dumps, error reports, emulator log folders. Files are MOVED to Quarantine (restorable for 7 days), never deleted straight away. The Recycle Bin is the only permanent item and is off by default.
- **Boost**: High Performance power plan, close chosen background programs (reopened on Stop), raise emulator priority. Security software, audio/graphics drivers, Discord, OBS and emulators are never closed.
- **Network**: quick lag check (ping, jitter, packet loss).
- **Startup**: list startup programs and switch them on or off.
- **History**: restore cleaned files, empty Quarantine, recent activity.
- **Settings / Feedback**: retention days, minimum file age, extra programs Boost may close, feedback web address.

## Safety rules (see core/src/safety.rs)
- Files newer than 24 hours are never cleaned.
- Every file is resolved (links and junctions) and must stay inside the folder being cleaned.
- Protected folders (for example BlueStacks `Engine`, LDPlayer `vms`, Gameloop `AOW_Rootfs`) are skipped.
- The UI only sends item ids, never paths.

## Get the Windows installer
1. Push this folder to the `main` branch of your GitHub repository.
2. Open **Actions > build-windows** (or press **Run workflow**).
3. Download the **EmuCleaner-installer** artifact and run the setup .exe.
4. The installer is unsigned, so SmartScreen warns. Choose **More info > Run anyway**.

## Feedback to a Google Sheet
See `docs/feedback-apps-script.gs`.

## Develop
- Core logic tests (Linux, macOS or Windows): `cd core && cargo test`
- UI preview with demo data: `cd ui && python3 -m http.server 8080`
- The real app builds on Windows only (use the GitHub Action).

## Layout
- `core/`       Rust logic: scan, clean, quarantine, boost, startup, network, settings (`emulators.json` lists the emulators)
- `src-tauri/`  Windows app shell (Tauri v2), a thin layer over `core`
- `ui/`         HTML, CSS and JavaScript interface
