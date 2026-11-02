use std::process::Command;

#[cfg(windows)]
fn hide(c: &mut Command) {
    use std::os::windows::process::CommandExt;
    c.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
}
#[cfg(not(windows))]
fn hide(_c: &mut Command) {}

/// Run a program and return its stdout. No console window is shown on Windows.
pub fn run(prog: &str, args: &[&str]) -> Result<String, String> {
    let mut c = Command::new(prog);
    c.args(args);
    hide(&mut c);
    let o = c.output().map_err(|e| format!("{}: {}", prog, e))?;
    if !o.status.success() {
        let err = String::from_utf8_lossy(&o.stderr).trim().to_string();
        let out = String::from_utf8_lossy(&o.stdout).trim().to_string();
        return Err(if err.is_empty() { out } else { err });
    }
    Ok(String::from_utf8_lossy(&o.stdout).into_owned())
}

pub fn powershell(script: &str) -> Result<String, String> {
    run("powershell", &["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", script])
}

/// Only plain process names may be passed into scripts.
pub fn safe_proc_name(n: &str) -> bool {
    !n.is_empty() && n.len() < 80 && n.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | ' '))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn proc_names() {
        assert!(safe_proc_name("HD-Player.exe"));
        assert!(!safe_proc_name("a'; calc"));
        assert!(!safe_proc_name(""));
    }
}
