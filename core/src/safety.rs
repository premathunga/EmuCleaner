use std::path::Path;

/// A file may be cleaned only if, after resolving links and junctions, it is still
/// inside `root` (already canonical) and no folder on its way is protected.
pub fn safe_file(file: &Path, root: &Path, protected: &[String]) -> bool {
    let Ok(c) = file.canonicalize() else { return false };
    if c == root || !c.starts_with(root) {
        return false;
    }
    let Ok(rel) = c.strip_prefix(root) else { return false };
    !rel.components().any(|comp| {
        let n = comp.as_os_str().to_string_lossy();
        protected.iter().any(|p| n.eq_ignore_ascii_case(p))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn inside_root_ok_protected_and_outside_rejected() {
        let t = tempfile::tempdir().unwrap();
        let root = t.path().join("root");
        fs::create_dir_all(root.join("vms")).unwrap();
        fs::create_dir_all(root.join("logs")).unwrap();
        fs::write(root.join("vms/a.vdi"), b"x").unwrap();
        fs::write(root.join("logs/a.log"), b"x").unwrap();
        fs::write(t.path().join("outside.txt"), b"x").unwrap();
        let rc = root.canonicalize().unwrap();
        let prot = vec!["vms".to_string()];
        assert!(safe_file(&root.join("logs/a.log"), &rc, &prot));
        assert!(!safe_file(&root.join("vms/a.vdi"), &rc, &prot));
        assert!(!safe_file(&t.path().join("outside.txt"), &rc, &prot));
        assert!(!safe_file(&root.join("missing.txt"), &rc, &prot));
    }

    #[cfg(unix)]
    #[test]
    fn symlink_escape_is_rejected() {
        let t = tempfile::tempdir().unwrap();
        let root = t.path().join("root");
        let other = t.path().join("other");
        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(&other).unwrap();
        fs::write(other.join("secret.txt"), b"x").unwrap();
        std::os::unix::fs::symlink(&other, root.join("link")).unwrap();
        let rc = root.canonicalize().unwrap();
        assert!(!safe_file(&root.join("link/secret.txt"), &rc, &[]));
    }
}
