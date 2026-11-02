use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct StartupItem {
    pub id: String,
    pub name: String,
    pub command: String,
    pub source: String,
    pub enabled: bool,
    /// "High", "Medium" or "Unknown" (a typical-impact estimate, not a measurement)
    pub impact: String,
    /// true for drivers, audio and security software
    pub keep: bool,
}

const KEEP: &[&str] = &[
    "securityhealth", "defender", "realtek", "rtkaud", "nvidia", "amd", "radeon", "intel", "synaptics", "touchpad", "bluetooth",
    "audio", "antivirus", "avast", "kaspersky", "eset", "mcafee", "norton", "bitdefender", "malwarebytes",
];
const HIGH: &[&str] = &[
    "discord", "steam", "teams", "onedrive", "adobe", "creative cloud", "spotify", "skype", "epicgames", "zoom", "dropbox", "slack",
    "googledrive", "anydesk", "overwolf", "origin", "ubisoft", "battle.net",
];
const MEDIUM: &[&str] = &["update", "updater", "launcher", "helper", "razer", "logitech", "corsair", "steelseries", "msedge", "chrome", "java"];

/// Typical impact and whether the item should normally be kept.
pub fn classify(name: &str, command: &str) -> (String, bool) {
    let t = format!("{} {}", name, command).to_lowercase();
    let keep = KEEP.iter().any(|k| t.contains(k));
    let impact = if keep {
        "Unknown"
    } else if HIGH.iter().any(|k| t.contains(k)) {
        "High"
    } else if MEDIUM.iter().any(|k| t.contains(k)) {
        "Medium"
    } else {
        "Unknown"
    };
    (impact.to_string(), keep)
}

/// StartupApproved values: first byte 02/06 = enabled, 03/07 = disabled.
pub fn approval_enabled(bytes: &[u8]) -> bool {
    bytes.first().map(|b| b % 2 == 0).unwrap_or(true)
}

pub fn approval_bytes(enabled: bool, now_unix_secs: u64) -> Vec<u8> {
    let mut v = vec![0u8; 12];
    if enabled {
        v[0] = 2;
    } else {
        v[0] = 3;
        let ft = (now_unix_secs + 11_644_473_600) * 10_000_000;
        v[4..12].copy_from_slice(&ft.to_le_bytes());
    }
    v
}

#[cfg(windows)]
mod win {
    use super::*;
    use winreg::enums::*;
    use winreg::{RegKey, RegValue};

    const RUN: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
    const RUN32: &str = r"Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Run";
    const OK_RUN: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run";
    const OK_FOLDER: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\StartupFolder";

    fn is_enabled(approved_path: &str, name: &str) -> bool {
        for hive in [HKEY_LOCAL_MACHINE, HKEY_CURRENT_USER] {
            if let Ok(k) = RegKey::predef(hive).open_subkey_with_flags(approved_path, KEY_READ) {
                if let Ok(v) = k.get_raw_value(name) {
                    if !approval_enabled(&v.bytes) {
                        return false;
                    }
                }
            }
        }
        true
    }

    fn folder() -> Option<std::path::PathBuf> {
        std::env::var("APPDATA").ok().map(|a| std::path::PathBuf::from(a).join(r"Microsoft\Windows\Start Menu\Programs\Startup"))
    }

    pub fn list() -> Vec<StartupItem> {
        let mut out = Vec::new();
        let sources = [("HKCU Run", HKEY_CURRENT_USER, RUN), ("HKLM Run", HKEY_LOCAL_MACHINE, RUN), ("HKLM Run 32-bit", HKEY_LOCAL_MACHINE, RUN32)];
        for (label, hive, path) in sources {
            if let Ok(k) = RegKey::predef(hive).open_subkey_with_flags(path, KEY_READ) {
                for r in k.enum_values() {
                    let Ok((name, _)) = r else { continue };
                    if name.is_empty() {
                        continue;
                    }
                    let cmd: String = k.get_value(&name).unwrap_or_default();
                    let (impact, keep) = classify(&name, &cmd);
                    out.push(StartupItem {
                        id: format!("{}|{}", label, name),
                        enabled: is_enabled(OK_RUN, &name),
                        name,
                        command: cmd,
                        source: label.to_string(),
                        impact,
                        keep,
                    });
                }
            }
        }
        if let Some(f) = folder() {
            for e in std::fs::read_dir(f).into_iter().flatten().flatten() {
                let name = e.file_name().to_string_lossy().into_owned();
                if name.eq_ignore_ascii_case("desktop.ini") {
                    continue;
                }
                let (impact, keep) = classify(&name, "");
                out.push(StartupItem {
                    id: format!("Startup folder|{}", name),
                    enabled: is_enabled(OK_FOLDER, &name),
                    command: e.path().to_string_lossy().into_owned(),
                    name,
                    source: "Startup folder".into(),
                    impact,
                    keep,
                });
            }
        }
        out
    }

    pub fn set_enabled(id: &str, enabled: bool) -> Result<(), String> {
        let (src, name) = id.split_once('|').ok_or("Invalid item")?;
        let approved = if src == "Startup folder" { OK_FOLDER } else { OK_RUN };
        if !list().iter().any(|i| i.id == id) {
            return Err("That startup item no longer exists.".into());
        }
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let val = RegValue { bytes: approval_bytes(enabled, now), vtype: REG_BINARY };
        let (k, _) = RegKey::predef(HKEY_CURRENT_USER).create_subkey_with_flags(approved, KEY_READ | KEY_WRITE).map_err(|e| e.to_string())?;
        k.set_raw_value(name, &val).map_err(|e| e.to_string())?;
        if src.starts_with("HKLM") {
            // machine-wide items may also need the machine-level approval (needs administrator)
            if let Ok((k2, _)) = RegKey::predef(HKEY_LOCAL_MACHINE).create_subkey_with_flags(approved, KEY_READ | KEY_WRITE) {
                let _ = k2.set_raw_value(name, &val);
            }
        }
        if is_enabled(approved, name) != enabled {
            return Err("Windows did not accept the change. Try running EmuCleaner as administrator.".into());
        }
        Ok(())
    }
}

#[cfg(windows)]
pub fn list() -> Result<Vec<StartupItem>, String> {
    let mut v = win::list();
    v.sort_by_key(|i| match i.impact.as_str() { "High" => 0, "Medium" => 1, _ => 2 });
    Ok(v)
}
#[cfg(windows)]
pub fn set_enabled(id: &str, enabled: bool) -> Result<(), String> {
    win::set_enabled(id, enabled)
}
#[cfg(not(windows))]
pub fn list() -> Result<Vec<StartupItem>, String> {
    Ok(vec![])
}
#[cfg(not(windows))]
pub fn set_enabled(_id: &str, _enabled: bool) -> Result<(), String> {
    Err("Startup management is only available on Windows.".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn approval_roundtrip() {
        assert!(approval_enabled(&approval_bytes(true, 1_700_000_000)));
        assert!(!approval_enabled(&approval_bytes(false, 1_700_000_000)));
        assert!(approval_enabled(&[6, 0, 0]));
        assert!(!approval_enabled(&[7, 0, 0]));
        assert_eq!(approval_bytes(false, 0).len(), 12);
    }
    #[test]
    fn classification() {
        assert_eq!(classify("Discord", "C:\\Update.exe").0, "High");
        assert!(classify("Realtek HD Audio Manager", "").1);
        assert_eq!(classify("MyTool", "x.exe").0, "Unknown");
        assert_eq!(classify("GoogleUpdateTaskMachine", "").0, "Medium");
    }
}
