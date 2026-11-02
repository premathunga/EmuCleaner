#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use emucleaner_core::{boost, history, junk, netcheck, quarantine, scan, settings, startup, sys};

// Every command runs on a worker thread so the window never freezes.
// All real logic (and its tests) lives in the `emucleaner-core` crate.
macro_rules! run {
    ($body:expr) => {
        tauri::async_runtime::spawn_blocking(move || $body)
            .await
            .map_err(|e| e.to_string())?
    };
}

#[tauri::command]
async fn system_stats() -> Result<sys::SystemStats, String> {
    Ok(run!(sys::read_stats()))
}

#[tauri::command]
async fn scan_emulators() -> Result<Vec<scan::EmulatorReport>, String> {
    Ok(run!(scan::scan_all()))
}

#[tauri::command]
async fn scan_junk() -> Result<Vec<junk::CleanItem>, String> {
    run!(junk::scan_junk())
}

#[tauri::command]
async fn clean_selected(ids: Vec<String>) -> Result<junk::CleanResult, String> {
    run!(junk::clean(&ids))
}

#[tauri::command]
async fn quarantine_list() -> Result<Vec<quarantine::BatchSummary>, String> {
    Ok(run!(quarantine::Store::default_store().list(settings::load().retention_days)))
}

#[tauri::command]
async fn quarantine_restore(id: String) -> Result<quarantine::RestoreResult, String> {
    run!({
        let r = quarantine::Store::default_store().restore(&id);
        if let Ok(x) = &r {
            history::add("restore", &format!("{} file(s) restored", x.restored), 0);
        }
        r
    })
}

#[tauri::command]
async fn quarantine_empty() -> Result<u64, String> {
    Ok(run!({
        let n = quarantine::Store::default_store().empty_all();
        history::add("empty", "Quarantine emptied", n);
        n
    }))
}

#[tauri::command]
async fn history_list() -> Result<Vec<history::Event>, String> {
    Ok(run!(history::list(30)))
}

#[tauri::command]
async fn boost_candidates() -> Result<Vec<boost::Candidate>, String> {
    run!(boost::candidates())
}

#[tauri::command]
async fn boost_start(power_plan: bool, priority: bool, close_apps: Vec<String>) -> Result<boost::BoostState, String> {
    run!(boost::start(power_plan, priority, close_apps))
}

#[tauri::command]
async fn boost_stop() -> Result<boost::BoostState, String> {
    run!(boost::stop())
}

#[tauri::command]
async fn boost_status() -> Result<boost::BoostState, String> {
    run!(boost::status())
}

#[tauri::command]
async fn boost_tick() -> Result<boost::BoostState, String> {
    run!(boost::tick())
}

#[tauri::command]
async fn startup_list() -> Result<Vec<startup::StartupItem>, String> {
    run!(startup::list())
}

#[tauri::command]
async fn startup_set(id: String, enabled: bool) -> Result<(), String> {
    run!(startup::set_enabled(&id, enabled))
}

#[tauri::command]
async fn lag_check() -> Result<netcheck::LagResult, String> {
    run!(netcheck::lag_check())
}

#[tauri::command]
async fn get_settings() -> Result<settings::Settings, String> {
    Ok(run!(settings::load()))
}

#[tauri::command]
async fn save_settings(value: settings::Settings) -> Result<settings::Settings, String> {
    run!(settings::save(&value))
}

fn main() {
    // remove quarantine batches older than the retention period
    std::thread::spawn(|| {
        quarantine::Store::default_store().purge_expired(settings::load().retention_days);
    });
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            system_stats,
            scan_emulators,
            scan_junk,
            clean_selected,
            quarantine_list,
            quarantine_restore,
            quarantine_empty,
            history_list,
            boost_candidates,
            boost_start,
            boost_stop,
            boost_status,
            boost_tick,
            startup_list,
            startup_set,
            lag_check,
            get_settings,
            save_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running EmuCleaner");
}
