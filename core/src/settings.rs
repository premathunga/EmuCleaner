use crate::paths;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// days cleaned files stay restorable
    pub retention_days: u32,
    /// files newer than this many hours are never cleaned
    pub min_age_hours: u64,
    /// extra program names (for example "myapp.exe") that Boost may close
    pub custom_close_apps: Vec<String>,
    /// Google Apps Script web app URL that receives feedback
    pub feedback_url: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { retention_days: 7, min_age_hours: 24, custom_close_apps: vec![], feedback_url: String::new() }
    }
}

pub fn load_from(dir: &Path) -> Settings {
    std::fs::read_to_string(dir.join("settings.json")).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
}

pub fn save_to(dir: &Path, s: &Settings) -> Result<(), String> {
    paths::ensure(dir)?;
    let mut s = s.clone();
    s.retention_days = s.retention_days.clamp(1, 60);
    s.min_age_hours = s.min_age_hours.min(24 * 30);
    s.custom_close_apps.retain(|n| crate::util::safe_proc_name(n));
    let json = serde_json::to_string_pretty(&s).map_err(|e| e.to_string())?;
    std::fs::write(dir.join("settings.json"), json).map_err(|e| e.to_string())
}

pub fn load() -> Settings {
    load_from(&paths::data_dir())
}

pub fn save(s: &Settings) -> Result<Settings, String> {
    save_to(&paths::data_dir(), s)?;
    Ok(load())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn roundtrip_and_clamp() {
        let t = tempfile::tempdir().unwrap();
        assert_eq!(load_from(t.path()).retention_days, 7);
        let s = Settings { retention_days: 999, custom_close_apps: vec!["ok.exe".into(), "bad';x".into()], ..Default::default() };
        save_to(t.path(), &s).unwrap();
        let l = load_from(t.path());
        assert_eq!(l.retention_days, 60);
        assert_eq!(l.custom_close_apps, vec!["ok.exe".to_string()]);
    }
}
