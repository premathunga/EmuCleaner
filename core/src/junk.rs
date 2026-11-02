use crate::{history, quarantine, safety, scan, settings, sys, util};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize)]
pub struct CleanItem {
    pub id: String,
    pub label: String,
    pub group: String,
    pub description: String,
    pub size_bytes: u64,
    pub files: u64,
    /// true = cannot be undone (Recycle Bin)
    pub permanent: bool,
    pub default_on: bool,
    pub warn_running: bool,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct CleanResult {
    pub batch_id: Option<String>,
    pub moved_bytes: u64,
    pub moved_files: u64,
    pub skipped_files: u64,
    pub permanent_bytes: u64,
    pub errors: Vec<String>,
}

struct Source {
    id: String,
    label: String,
    group: String,
    description: String,
    root: PathBuf,
    permanent: bool,
    default_on: bool,
    protected: Vec<String>,
    emulator_procs: Vec<String>,
}

fn sources() -> Vec<Source> {
    let mk = |id: &str, label: &str, desc: &str, root: &str, on: bool| Source {
        id: id.into(),
        label: label.into(),
        group: "Windows".into(),
        description: desc.into(),
        root: PathBuf::from(scan::expand(root)),
        permanent: false,
        default_on: on,
        protected: vec![],
        emulator_procs: vec![],
    };
    let mut v = vec![
        mk("win_temp", "Temporary files", "Files apps left in your Temp folder", "%TEMP%", true),
        mk("sys_temp", "Windows temporary files", "Files in the Windows Temp folder", "%WINDIR%\\Temp", true),
        mk("crash_dumps", "Crash dumps", "Memory dumps from crashed apps", "%LOCALAPPDATA%\\CrashDumps", true),
        mk("wer", "Error reports", "Windows error report files", "%LOCALAPPDATA%\\Microsoft\\Windows\\WER", true),
    ];
    let mut rb = mk("recycle", "Recycle Bin", "Files you already deleted. Emptying it cannot be undone", "%SystemDrive%\\$Recycle.Bin", false);
    rb.permanent = true;
    v.push(rb);
    for d in scan::load_defs() {
        for c in &d.candidates {
            if c.kind == "logs" || c.kind == "temp" {
                v.push(Source {
                    id: format!("emu:{}:{}", d.id, c.label.to_lowercase().replace(' ', "_")),
                    label: c.label.clone(),
                    group: d.name.clone(),
                    description: format!("{} files of {}", c.kind, d.name),
                    root: PathBuf::from(scan::expand(&c.path)),
                    permanent: false,
                    default_on: true,
                    protected: d.protected.clone(),
                    emulator_procs: d.processes.clone(),
                });
            }
        }
    }
    v
}

/// Files below `root` that are old enough and pass every safety check.
pub fn collect(root: &Path, min_age: Duration, protected: &[String]) -> Vec<(PathBuf, u64)> {
    let Ok(rc) = root.canonicalize() else { return vec![] };
    let now = SystemTime::now();
    let mut out = Vec::new();
    for e in WalkDir::new(&rc).follow_links(false).into_iter().filter_map(|e| e.ok()) {
        if !e.file_type().is_file() {
            continue;
        }
        let Ok(md) = e.metadata() else { continue };
        let old = md.modified().ok().and_then(|m| now.duration_since(m).ok()).map(|d| d >= min_age).unwrap_or(false);
        if !old || !safety::safe_file(e.path(), &rc, protected) {
            continue;
        }
        out.push((e.path().to_path_buf(), md.len()));
    }
    out
}

pub fn scan_junk() -> Result<Vec<CleanItem>, String> {
    let st = settings::load();
    let min_age = Duration::from_secs(st.min_age_hours * 3600);
    let running = sys::running_process_names();
    let mut items = Vec::new();
    for s in sources() {
        if !s.root.is_dir() {
            continue;
        }
        let (size, files) = if s.permanent {
            (scan::dir_size(&s.root), 0)
        } else {
            let f = collect(&s.root, min_age, &s.protected);
            (f.iter().map(|x| x.1).sum(), f.len() as u64)
        };
        if size == 0 {
            continue;
        }
        items.push(CleanItem {
            id: s.id.clone(),
            label: s.label.clone(),
            group: s.group.clone(),
            description: s.description.clone(),
            size_bytes: size,
            files,
            permanent: s.permanent,
            default_on: s.default_on,
            warn_running: s.emulator_procs.iter().any(|p| running.contains(&p.to_lowercase())),
        });
    }
    Ok(items)
}

fn empty_recycle_bin() -> Result<(), String> {
    if !cfg!(windows) {
        return Err("Windows only".into());
    }
    util::powershell("Clear-RecycleBin -Force -ErrorAction SilentlyContinue").map(|_| ())
}

/// Move the selected items to quarantine. Ids are looked up on our side, never trusted as paths.
pub fn clean(ids: &[String]) -> Result<CleanResult, String> {
    let st = settings::load();
    let min_age = Duration::from_secs(st.min_age_hours * 3600);
    let srcs = sources();
    let store = quarantine::Store::default_store();
    let mut batch = store.begin("Junk cleanup")?;
    let mut res = CleanResult::default();
    for id in ids {
        let Some(s) = srcs.iter().find(|s| &s.id == id) else {
            res.errors.push(format!("Unknown item: {}", id));
            continue;
        };
        if s.permanent {
            let before = scan::dir_size(&s.root);
            match empty_recycle_bin() {
                Ok(()) => res.permanent_bytes += before,
                Err(e) => res.errors.push(format!("{}: {}", s.label, e)),
            }
            continue;
        }
        for (p, len) in collect(&s.root, min_age, &s.protected) {
            match batch.add(&p) {
                Ok(_) => {
                    res.moved_bytes += len;
                    res.moved_files += 1;
                }
                Err(_) => res.skipped_files += 1,
            }
        }
    }
    let m = batch.finish()?;
    if !m.entries.is_empty() {
        res.batch_id = Some(m.id);
    }
    store.purge_expired(st.retention_days);
    let total = res.moved_bytes + res.permanent_bytes;
    history::add("clean", &format!("{} files cleaned", res.moved_files), total);
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn collect_respects_age_and_protected() {
        let t = tempfile::tempdir().unwrap();
        let root = t.path().join("r");
        fs::create_dir_all(root.join("keep")).unwrap();
        fs::write(root.join("a.tmp"), vec![0u8; 10]).unwrap();
        fs::write(root.join("keep/b.tmp"), vec![0u8; 20]).unwrap();
        let prot = vec!["keep".to_string()];
        let new_skipped = collect(&root, Duration::from_secs(3600), &prot);
        assert!(new_skipped.is_empty(), "fresh files must not be cleaned");
        let all = collect(&root, Duration::ZERO, &prot);
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].1, 10);
    }

    #[test]
    fn sources_have_unique_ids() {
        let s = sources();
        let mut ids: Vec<_> = s.iter().map(|x| x.id.clone()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), s.len());
        assert!(s.iter().any(|x| x.permanent && !x.default_on));
    }
}
