# EmuCleaner (stage 1)

Stage 1 is READ-ONLY. It measures RAM, CPU and disk, finds BlueStacks / LDPlayer / Gameloop,
and shows folder sizes. It deletes and changes nothing.

## Get the Windows installer
1. Create a GitHub repository and push this folder to the `main` branch.
2. Open the repository, go to **Actions**, then **build-windows** (or press **Run workflow**).
3. When the run is green, open it and download **EmuCleaner-installer** (setup .exe).
4. The installer is unsigned, so Windows SmartScreen shows a warning. Choose **More info**, then **Run anyway**.

## Test it
1. Install and open EmuCleaner on a Windows PC that has an emulator installed.
2. Press **Copy scan report** and paste the text to the developer.
3. Tell the developer if anything looks wrong (missing emulator, wrong sizes, crashes).

## Develop (GitHub Codespaces or any Linux/macOS machine)
- Core logic tests: `cd core && cargo test`
- Preview the UI with demo data: `cd ui && python3 -m http.server 8080` (open the forwarded port)
- The real app only builds on Windows (use the GitHub Action).

## Layout
- `core/`       cross-platform Rust logic (scan, score, emulator definitions in `emulators.json`)
- `src-tauri/`  Windows app shell (Tauri v2)
- `ui/`         HTML/CSS/JS interface
