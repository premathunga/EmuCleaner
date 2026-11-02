use serde::Serialize;
use std::collections::HashSet;
use sysinfo::{Disks, System, MINIMUM_CPU_UPDATE_INTERVAL};

#[derive(Debug, Clone, Serialize)]
pub struct SystemStats {
    pub ram_total: u64,
    pub ram_free: u64,
    pub cpu_percent: f32,
    pub disk_total: u64,
    pub disk_free: u64,
    pub score: u8,
}

/// Game Ready Score (0-100). Honest, simple formula based only on measured values:
///   40%  free RAM   (50% or more free counts as full marks)
///   30%  free disk  (30% or more free on the system drive counts as full marks)
///   30%  idle CPU   (100% minus current CPU load)
pub fn compute_score(ram_free_ratio: f64, disk_free_ratio: f64, cpu_percent: f64) -> u8 {
    let ram = (ram_free_ratio / 0.5).clamp(0.0, 1.0);
    let disk = (disk_free_ratio / 0.3).clamp(0.0, 1.0);
    let cpu = (1.0 - cpu_percent / 100.0).clamp(0.0, 1.0);
    (100.0 * (0.4 * ram + 0.3 * disk + 0.3 * cpu)).round() as u8
}

pub fn read_stats() -> SystemStats {
    let mut sys = System::new();
    sys.refresh_memory();
    sys.refresh_cpu();
    std::thread::sleep(MINIMUM_CPU_UPDATE_INTERVAL);
    sys.refresh_cpu();
    let cpu = sys.global_cpu_info().cpu_usage();
    let ram_total = sys.total_memory();
    let ram_free = sys.available_memory();

    let disks = Disks::new_with_refreshed_list();
    let sysdrive = std::env::var("SystemDrive").unwrap_or_else(|_| "C:".into()).to_uppercase();
    let pick = disks
        .list()
        .iter()
        .find(|d| d.mount_point().to_string_lossy().to_uppercase().starts_with(&sysdrive))
        .or_else(|| disks.list().iter().max_by_key(|d| d.total_space()));
    let (disk_total, disk_free) = pick.map(|d| (d.total_space(), d.available_space())).unwrap_or((0, 0));

    let ratio = |a: u64, b: u64| if b == 0 { 0.0 } else { a as f64 / b as f64 };
    let score = compute_score(ratio(ram_free, ram_total), ratio(disk_free, disk_total), cpu as f64);
    SystemStats { ram_total, ram_free, cpu_percent: cpu, disk_total, disk_free, score }
}

/// Lower-cased names of all running processes.
pub fn running_process_names() -> HashSet<String> {
    let mut sys = System::new();
    sys.refresh_processes();
    sys.processes().values().map(|p| p.name().to_lowercase()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn score_bounds() {
        assert_eq!(compute_score(1.0, 1.0, 0.0), 100);
        assert_eq!(compute_score(0.0, 0.0, 100.0), 0);
        assert_eq!(compute_score(0.5, 0.3, 0.0), 100);
    }
    #[test]
    fn score_is_monotonic_in_ram() {
        assert!(compute_score(0.2, 0.5, 30.0) < compute_score(0.4, 0.5, 30.0));
    }
    #[test]
    fn stats_do_not_panic() {
        let s = read_stats();
        assert!(s.score <= 100);
    }
}
