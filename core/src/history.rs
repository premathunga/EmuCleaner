use crate::paths;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub ts_ms: u64,
    pub kind: String,
    pub text: String,
    pub bytes: u64,
}

pub fn add_to(dir: &Path, kind: &str, text: &str, bytes: u64) {
    let mut v = list_from(dir, 1000);
    v.insert(0, Event { ts_ms: paths::now_ms(), kind: kind.into(), text: text.into(), bytes });
    v.truncate(200);
    if paths::ensure(dir).is_ok() {
        let _ = std::fs::write(dir.join("history.json"), serde_json::to_string(&v).unwrap_or_default());
    }
}

pub fn list_from(dir: &Path, limit: usize) -> Vec<Event> {
    let mut v: Vec<Event> =
        std::fs::read_to_string(dir.join("history.json")).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default();
    v.truncate(limit);
    v
}

pub fn add(kind: &str, text: &str, bytes: u64) {
    add_to(&paths::data_dir(), kind, text, bytes)
}

pub fn list(limit: usize) -> Vec<Event> {
    list_from(&paths::data_dir(), limit)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn newest_first_and_capped() {
        let t = tempfile::tempdir().unwrap();
        add_to(t.path(), "clean", "a", 1);
        add_to(t.path(), "boost", "b", 0);
        let l = list_from(t.path(), 10);
        assert_eq!(l[0].kind, "boost");
        assert_eq!(l.len(), 2);
    }
}
