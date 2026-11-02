#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use emucleaner_core::{scan, sys};

// Stage 1 is read-only: these commands only measure things. Nothing is deleted or changed.

#[tauri::command]
async fn system_stats() -> Result<sys::SystemStats, String> {
    tauri::async_runtime::spawn_blocking(sys::read_stats)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn scan_emulators() -> Result<Vec<scan::EmulatorReport>, String> {
    tauri::async_runtime::spawn_blocking(scan::scan_all)
        .await
        .map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![system_stats, scan_emulators])
        .run(tauri::generate_context!())
        .expect("error while running EmuCleaner");
}
