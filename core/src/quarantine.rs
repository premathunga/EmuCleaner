use crate::paths;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub orig: String,
    pub stored: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub id: String,
    pub created_ms: u64,
    pub label: String,
    pub total_bytes: u64,
    pub entries: Vec<Entry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BatchSummary {
    pub id: String,
    pub created_ms: u64,
    pub label: String,
    pub total_bytes: u64,
    pub files: usize,
    pub days_left: i64,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct RestoreResult {
    pub restored: usize,
    pub skipped: usize,
}

const DAY_MS: u64 = 86_400_000;

pub struct Store {
    root: PathBuf,
}

pub struct Batch {
    dir: PathBuf,
    manifest: Manifest,
    next: u64,
}

fn move_file(from: &Path, to: &Path) -> Result<(), String> {
    if fs::rename(from, to).is_ok() {
        return Ok(());
    }
    // different drive: copy, then remove the original
    fs::copy(from, to).map_err(|e| e.to_string())?;
    if let Err(e) = fs::remove_file(from) {
        let _ = fs::remove_file(to);
        return Err(e.to_string());
    }
    Ok(())
}

impl Store {
    pub fn new(root: PathBuf) -> Self {
        Store { root }
    }
    pub fn default_store() -> Self {
        Store::new(paths::quarantine_dir())
    }

    pub fn begin(&self, label: &str) -> Result<Batch, String> {
        self.begin_at(label, paths::now_ms())
    }

    pub fn begin_at(&self, label: &str, created_ms: u64) -> Result<Batch, String> {
        let mut id = created_ms.to_string();
        let mut n = 1;
        while self.root.join(&id).exists() {
            id = format!("{}-{}", created_ms, n);
            n += 1;
        }
        let dir = self.root.join(&id);
        paths::ensure(&dir.join("files"))?;
        Ok(Batch {
            dir,
            manifest: Manifest { id, created_ms, label: label.to_string(), total_bytes: 0, entries: vec![] },
            next: 0,
        })
    }

    fn read_manifest(dir: &Path) -> Option<Manifest> {
        serde_json::from_str(&fs::read_to_string(dir.join("manifest.json")).ok()?).ok()
    }

    pub fn list(&self, retention_days: u32) -> Vec<BatchSummary> {
        let now = paths::now_ms();
        let mut v: Vec<BatchSummary> = fs::read_dir(&self.root)
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|e| Self::read_manifest(&e.path()))
            .map(|m| {
                let age_days = (now.saturating_sub(m.created_ms) / DAY_MS) as i64;
                BatchSummary {
                    id: m.id.clone(),
                    created_ms: m.created_ms,
                    label: m.label.clone(),
                    total_bytes: m.entries.iter().map(|e| e.size).sum(),
                    files: m.entries.len(),
                    days_left: (retention_days as i64 - age_days).max(0),
                }
            })
            .collect();
        v.sort_by(|a, b| b.created_ms.cmp(&a.created_ms));
        v
    }

    pub fn restore(&self, id: &str) -> Result<RestoreResult, String> {
        if id.is_empty() || !id.chars().all(|c| c.is_ascii_digit() || c == '-') {
            return Err("Invalid batch id".into());
        }
        let dir = self.root.join(id);
        let mut m = Self::read_manifest(&dir).ok_or("Quarantine batch not found")?;
        let mut res = RestoreResult::default();
        let mut remaining = Vec::new();
        for e in m.entries.drain(..) {
            let orig = PathBuf::from(&e.orig);
            let stored = PathBuf::from(&e.stored);
            if orig.exists() || !stored.exists() {
                res.skipped += 1;
                remaining.push(e);
                continue;
            }
            if let Some(p) = orig.parent() {
                let _ = fs::create_dir_all(p);
            }
            match move_file(&stored, &orig) {
                Ok(()) => res.restored += 1,
                Err(_) => {
                    res.skipped += 1;
                    remaining.push(e);
                }
            }
        }
        if remaining.is_empty() {
            let _ = fs::remove_dir_all(&dir);
        } else {
            m.entries = remaining;
            let _ = fs::write(dir.join("manifest.json"), serde_json::to_string_pretty(&m).unwrap_or_default());
        }
        Ok(res)
    }

    /// Permanently delete batches older than `retention_days`. Returns freed bytes.
    pub fn purge_expired(&self, retention_days: u32) -> u64 {
        let now = paths::now_ms();
        let mut freed = 0;
        for e in fs::read_dir(&self.root).into_iter().flatten().flatten() {
            if let Some(m) = Self::read_manifest(&e.path()) {
                if now.saturating_sub(m.created_ms) / DAY_MS >= retention_days as u64 {
                    freed += m.entries.iter().map(|x| x.size).sum::<u64>();
                    let _ = fs::remove_dir_all(e.path());
                }
            }
        }
        freed
    }

    /// Permanently delete everything in quarantine. Returns freed bytes.
    pub fn empty_all(&self) -> u64 {
        let freed = self.total_bytes();
        for e in fs::read_dir(&self.root).into_iter().flatten().flatten() {
            let _ = fs::remove_dir_all(e.path());
        }
        freed
    }

    pub fn total_bytes(&self) -> u64 {
        fs::read_dir(&self.root)
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|e| Self::read_manifest(&e.path()))
            .map(|m| m.entries.iter().map(|x| x.size).sum::<u64>())
            .sum()
    }
}

impl Batch {
    /// Move one file into this batch. Fails (and leaves the file alone) if it is in use.
    pub fn add(&mut self, orig: &Path) -> Result<u64, String> {
        let size = fs::metadata(orig).map_err(|e| e.to_string())?.len();
        let stored = self.dir.join("files").join(self.next.to_string());
        move_file(orig, &stored)?;
        self.next += 1;
        self.manifest.total_bytes += size;
        self.manifest.entries.push(Entry {
            orig: orig.to_string_lossy().into_owned(),
            stored: stored.to_string_lossy().into_owned(),
            size,
        });
        Ok(size)
    }

    pub fn finish(self) -> Result<Manifest, String> {
        if self.manifest.entries.is_empty() {
            let _ = fs::remove_dir_all(&self.dir);
            return Ok(self.manifest);
        }
        let json = serde_json::to_string_pretty(&self.manifest).map_err(|e| e.to_string())?;
        fs::write(self.dir.join("manifest.json"), json).map_err(|e| e.to_string())?;
        Ok(self.manifest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (tempfile::TempDir, Store, PathBuf) {
        let t = tempfile::tempdir().unwrap();
        let store = Store::new(t.path().join("q"));
        let src = t.path().join("src");
        fs::create_dir_all(&src).unwrap();
        (t, store, src)
    }

    #[test]
    fn move_list_restore() {
        let (_t, store, src) = setup();
        let f = src.join("a.log");
        fs::write(&f, vec![1u8; 100]).unwrap();
        let mut b = store.begin("test").unwrap();
        assert_eq!(b.add(&f).unwrap(), 100);
        let m = b.finish().unwrap();
        assert!(!f.exists());
        let l = store.list(7);
        assert_eq!(l.len(), 1);
        assert_eq!(l[0].files, 1);
        assert_eq!(l[0].total_bytes, 100);
        assert_eq!(l[0].days_left, 7);
        let r = store.restore(&m.id).unwrap();
        assert_eq!(r.restored, 1);
        assert!(f.exists());
        assert!(store.list(7).is_empty());
    }

    #[test]
    fn restore_skips_when_original_exists() {
        let (_t, store, src) = setup();
        let f = src.join("a.log");
        fs::write(&f, b"old").unwrap();
        let mut b = store.begin("t").unwrap();
        b.add(&f).unwrap();
        let m = b.finish().unwrap();
        fs::write(&f, b"new").unwrap();
        let r = store.restore(&m.id).unwrap();
        assert_eq!((r.restored, r.skipped), (0, 1));
        assert_eq!(fs::read(&f).unwrap(), b"new");
        assert_eq!(store.list(7).len(), 1);
    }

    #[test]
    fn empty_batch_leaves_nothing() {
        let (_t, store, _src) = setup();
        let b = store.begin("t").unwrap();
        b.finish().unwrap();
        assert!(store.list(7).is_empty());
    }

    #[test]
    fn purge_and_empty() {
        let (_t, store, src) = setup();
        let old = paths::now_ms() - 10 * DAY_MS;
        let f1 = src.join("old.log");
        fs::write(&f1, vec![0u8; 50]).unwrap();
        let mut b = store.begin_at("old", old).unwrap();
        b.add(&f1).unwrap();
        b.finish().unwrap();
        let f2 = src.join("new.log");
        fs::write(&f2, vec![0u8; 70]).unwrap();
        let mut b = store.begin("new").unwrap();
        b.add(&f2).unwrap();
        b.finish().unwrap();
        assert_eq!(store.purge_expired(7), 50);
        assert_eq!(store.list(7).len(), 1);
        assert_eq!(store.total_bytes(), 70);
        assert_eq!(store.empty_all(), 70);
        assert!(store.list(7).is_empty());
    }

    #[test]
    fn rejects_bad_ids() {
        let (_t, store, _s) = setup();
        assert!(store.restore("../etc").is_err());
    }
}
