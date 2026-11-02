use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const DEFS_JSON: &str = include_str!("../emulators.json");

#[derive(Debug, Clone, Deserialize)]
pub struct RegistrySpec {
    pub key: String,
    pub values: Vec<String>,
    #[serde(default)]
    pub parent: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CandidateDef {
    pub label: String,
    pub kind: String,
    pub path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmulatorDef {
    pub id: String,
    pub name: String,
    pub processes: Vec<String>,
    pub install_dirs: Vec<String>,
    pub data_dirs: Vec<String>,
    #[serde(default)]
    pub registry: Vec<RegistrySpec>,
    #[serde(default)]
    pub candidates: Vec<CandidateDef>,
    #[serde(default)]
    pub protected: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CandidateReport {
    pub label: String,
    pub kind: String,
    pub path: String,
    pub exists: bool,
    pub size_bytes: u64,
    /// false until the folder has been confirmed safe on a real install
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct FolderReport {
    pub path: String,
    pub size_bytes: u64,
    pub protected: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct EmulatorReport {
    pub id: String,
    pub name: String,
    pub installed: bool,
    pub running: bool,
    pub found_dirs: Vec<String>,
    pub total_bytes: u64,
    pub candidates: Vec<CandidateReport>,
    pub breakdown: Vec<FolderReport>,
}

pub fn load_defs() -> Vec<EmulatorDef> {
    serde_json::from_str(DEFS_JSON).expect("emulators.json must be valid")
}

/// Replace %VAR% with environment values and use the separators of the current OS.
pub fn expand(p: &str) -> String {
    let mut out = String::new();
    let mut rest = p;
    while let Some(i) = rest.find('%') {
        out.push_str(&rest[..i]);
        let after = &rest[i + 1..];
        match after.find('%') {
            Some(j) => {
                let name = &after[..j];
                match std::env::var(name) {
                    Ok(v) => out.push_str(&v),
                    Err(_) => {
                        out.push('%');
                        out.push_str(name);
                        out.push('%');
                    }
                }
                rest = &after[j + 1..];
            }
            None => {
                out.push('%');
                rest = after;
                break;
            }
        }
    }
    out.push_str(rest);
    if std::path::MAIN_SEPARATOR == '/' {
        out.replace('\\', "/")
    } else {
        out
    }
}

/// Total size of all files below `p` (symlinks are not followed, errors are ignored).
pub fn dir_size(p: &Path) -> u64 {
    WalkDir::new(p)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.metadata().map(|m| m.len()).unwrap_or(0))
        .sum()
}

fn lower(p: &Path) -> PathBuf {
    PathBuf::from(p.to_string_lossy().to_lowercase())
}

/// Remove duplicates and folders that live inside another folder of the list.
pub fn dedupe_nested(mut v: Vec<PathBuf>) -> Vec<PathBuf> {
    v.sort_by_key(|p| p.components().count());
    let mut kept: Vec<PathBuf> = Vec::new();
    for p in v {
        let lp = lower(&p);
        if !kept.iter().any(|k| lp.starts_with(lower(k))) {
            kept.push(p);
        }
    }
    kept
}

#[cfg(windows)]
fn registry_dirs(specs: &[RegistrySpec]) -> Vec<PathBuf> {
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_32KEY, KEY_WOW64_64KEY};
    use winreg::RegKey;
    let mut out = Vec::new();
    for s in specs {
        for hive in [HKEY_LOCAL_MACHINE, HKEY_CURRENT_USER] {
            for view in [KEY_WOW64_64KEY, KEY_WOW64_32KEY] {
                if let Ok(key) = RegKey::predef(hive).open_subkey_with_flags(&s.key, KEY_READ | view) {
                    for name in &s.values {
                        if let Ok(v) = key.get_value::<String, _>(name) {
                            let p = PathBuf::from(v);
                            if s.parent {
                                if let Some(par) = p.parent() {
                                    out.push(par.to_path_buf());
                                }
                            }
                            out.push(p);
                        }
                    }
                }
            }
        }
    }
    out
}

#[cfg(not(windows))]
fn registry_dirs(_specs: &[RegistrySpec]) -> Vec<PathBuf> {
    Vec::new()
}

pub fn scan_def(def: &EmulatorDef, running: &HashSet<String>) -> EmulatorReport {
    let mut found: Vec<PathBuf> = def
        .install_dirs
        .iter()
        .chain(def.data_dirs.iter())
        .map(|d| PathBuf::from(expand(d)))
        .collect();
    found.extend(registry_dirs(&def.registry));
    found.retain(|p| p.is_dir());
    let roots = dedupe_nested(found);

    let total_bytes = roots.iter().map(|p| dir_size(p)).sum();
    let is_running = def.processes.iter().any(|p| running.contains(&p.to_lowercase()));

    let candidates = def
        .candidates
        .iter()
        .map(|c| {
            let p = PathBuf::from(expand(&c.path));
            let exists = p.is_dir();
            CandidateReport {
                label: c.label.clone(),
                kind: c.kind.clone(),
                path: p.to_string_lossy().into_owned(),
                exists,
                size_bytes: if exists { dir_size(&p) } else { 0 },
                verified: false,
            }
        })
        .collect();

    let mut breakdown: Vec<FolderReport> = Vec::new();
    for root in &roots {
        if let Ok(rd) = std::fs::read_dir(root) {
            for entry in rd.filter_map(|e| e.ok()) {
                let path = entry.path();
                let size = if path.is_dir() {
                    dir_size(&path)
                } else {
                    entry.metadata().map(|m| m.len()).unwrap_or(0)
                };
                let name = entry.file_name().to_string_lossy().into_owned();
                let protected = def.protected.iter().any(|n| n.eq_ignore_ascii_case(&name));
                breakdown.push(FolderReport { path: path.to_string_lossy().into_owned(), size_bytes: size, protected });
            }
        }
    }
    breakdown.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
    breakdown.truncate(15);

    EmulatorReport {
        id: def.id.clone(),
        name: def.name.clone(),
        installed: !roots.is_empty(),
        running: is_running,
        found_dirs: roots.iter().map(|p| p.to_string_lossy().into_owned()).collect(),
        total_bytes,
        candidates,
        breakdown,
    }
}

pub fn scan_all() -> Vec<EmulatorReport> {
    let running = crate::sys::running_process_names();
    load_defs().iter().map(|d| scan_def(d, &running)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn def(root: &str) -> EmulatorDef {
        EmulatorDef {
            id: "t".into(),
            name: "Test".into(),
            processes: vec!["Test.exe".into()],
            install_dirs: vec![format!("%{}%\\Install", root)],
            data_dirs: vec![format!("%{}%\\Install\\Data", root), format!("%{}%\\Missing", root)],
            registry: vec![],
            candidates: vec![CandidateDef { label: "Logs".into(), kind: "logs".into(), path: format!("%{}%\\Install\\Logs", root) }],
            protected: vec!["vms".into()],
        }
    }

    #[test]
    fn defs_file_is_valid() {
        let d = load_defs();
        assert!(d.len() >= 3);
        assert!(d.iter().all(|x| !x.processes.is_empty()));
    }

    #[test]
    fn expand_known_and_unknown_vars() {
        std::env::set_var("EMUC_T1", "/tmp/x");
        assert_eq!(expand("%EMUC_T1%\\a\\b"), "/tmp/x/a/b");
        assert!(expand("%EMUC_NOPE%\\a").contains("%EMUC_NOPE%"));
        assert_eq!(expand("50% off"), "50% off");
    }

    #[test]
    fn scan_finds_sizes_protected_and_running() {
        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path().join("Install");
        fs::create_dir_all(base.join("vms")).unwrap();
        fs::create_dir_all(base.join("Logs")).unwrap();
        fs::create_dir_all(base.join("Data")).unwrap();
        fs::write(base.join("vms").join("disk.vdi"), vec![0u8; 5000]).unwrap();
        fs::write(base.join("Logs").join("a.log"), vec![0u8; 700]).unwrap();
        fs::write(base.join("Data").join("b.dat"), vec![0u8; 300]).unwrap();
        std::env::set_var("EMUC_T2", tmp.path());

        let mut running = HashSet::new();
        running.insert("test.exe".to_string());
        let r = scan_def(&def("EMUC_T2"), &running);

        assert!(r.installed && r.running);
        assert_eq!(r.found_dirs.len(), 1, "nested Data folder must not be counted twice");
        assert_eq!(r.total_bytes, 6000);
        assert_eq!(r.candidates[0].size_bytes, 700);
        assert!(r.breakdown.iter().any(|f| f.protected && f.path.ends_with("vms")));
        assert!(r.breakdown[0].size_bytes >= r.breakdown[1].size_bytes);
    }

    #[test]
    fn missing_install_is_not_installed() {
        std::env::set_var("EMUC_T3", "/definitely/not/here");
        let r = scan_def(&def("EMUC_T3"), &HashSet::new());
        assert!(!r.installed && !r.running && r.total_bytes == 0);
    }

    #[test]
    fn dedupe_removes_nested_and_duplicates() {
        let v = vec![PathBuf::from("/a/b/c"), PathBuf::from("/a/b"), PathBuf::from("/A/B"), PathBuf::from("/x")];
        let k = dedupe_nested(v);
        assert_eq!(k.len(), 2);
    }
}
