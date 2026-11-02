//! End-to-end test of the cleaning flow with real files (Linux/Windows).
use emucleaner_core::{history, junk, quarantine, settings};
use std::fs;

#[test]
fn scan_clean_restore_flow() {
    let t = tempfile::tempdir().unwrap();
    let data = t.path().join("data");
    let temp = t.path().join("temp");
    fs::create_dir_all(&temp).unwrap();
    std::env::set_var("EMUCLEANER_DATA", &data);
    std::env::set_var("TEMP", &temp);
    settings::save(&settings::Settings { min_age_hours: 0, ..Default::default() }).unwrap();

    fs::write(temp.join("a.tmp"), vec![0u8; 1000]).unwrap();
    fs::create_dir_all(temp.join("sub")).unwrap();
    fs::write(temp.join("sub/b.tmp"), vec![0u8; 500]).unwrap();

    let items = junk::scan_junk().unwrap();
    let it = items.iter().find(|i| i.id == "win_temp").expect("win_temp item");
    assert_eq!((it.size_bytes, it.files), (1500, 2));
    assert!(!it.permanent);

    // unknown ids are rejected, known ones are cleaned
    let res = junk::clean(&["win_temp".into(), "../../etc".into()]).unwrap();
    assert_eq!((res.moved_files, res.moved_bytes), (2, 1500));
    assert_eq!(res.errors.len(), 1);
    assert!(!temp.join("a.tmp").exists());

    let store = quarantine::Store::default_store();
    let list = store.list(7);
    assert_eq!(list.len(), 1);
    let r = store.restore(&list[0].id).unwrap();
    assert_eq!(r.restored, 2);
    assert!(temp.join("a.tmp").exists() && temp.join("sub/b.tmp").exists());
    assert_eq!(history::list(5)[0].kind, "clean");
}
