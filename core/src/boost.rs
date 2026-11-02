use crate::{history, paths, scan, util};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use sysinfo::System;

const HIGH_PERFORMANCE: &str = "8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c";

/// (process name, display name, category, ticked by default)
const KNOWN: &[(&str, &str, &str, bool)] = &[
    ("onedrive.exe", "OneDrive", "Cloud sync", true),
    ("dropbox.exe", "Dropbox", "Cloud sync", true),
    ("googledrivefs.exe", "Google Drive", "Cloud sync", true),
    ("googleupdate.exe", "Google Updater", "Updater", true),
    ("microsoftedgeupdate.exe", "Edge Updater", "Updater", true),
    ("jusched.exe", "Java Updater", "Updater", true),
    ("adobeupdateservice.exe", "Adobe Updater", "Updater", true),
    ("adobe desktop service.exe", "Adobe Desktop Service", "Updater", true),
    ("ccxprocess.exe", "Adobe Creative Cloud helper", "Updater", true),
    ("teams.exe", "Microsoft Teams", "Chat", false),
    ("ms-teams.exe", "Microsoft Teams", "Chat", false),
    ("skype.exe", "Skype", "Chat", false),
    ("slack.exe", "Slack", "Chat", false),
    ("zoom.exe", "Zoom", "Chat", false),
    ("spotify.exe", "Spotify", "Music", false),
    ("epicgameslauncher.exe", "Epic Games Launcher", "Launcher", false),
    ("steam.exe", "Steam", "Launcher", false),
];

/// Programs that are never closed, even if the user adds them to the list.
const PROTECTED_PREFIXES: &[&str] = &[
    "system", "smss", "csrss", "wininit", "winlogon", "services", "lsass", "svchost", "dwm", "explorer", "sihost", "taskhost",
    "ctfmon", "fontdrvhost", "runtimebroker", "searchhost", "searchindexer", "startmenuexperiencehost", "shellexperiencehost",
    "textinputhost", "audiodg", "msmpeng", "nissrv", "securityhealth", "windefend", "avast", "avg", "kaspersky", "bitdefender",
    "mcafee", "norton", "eset", "malwarebytes", "rtk", "realtek", "nvidia", "nvcontainer", "amd", "radeon", "intel", "obs",
    "discord", "emucleaner", "msedgewebview2", "hd-", "bluestacks", "dnplayer", "ld9", "ldvbox", "aow_", "androidemulator",
    "appmarket", "qmemulator", "tbswebrenderer", "powershell", "cmd", "conhost", "wudfhost",
];

pub fn is_protected(name: &str) -> bool {
    let n = name.to_lowercase();
    PROTECTED_PREFIXES.iter().any(|p| n.starts_with(p))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosedApp {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct BoostState {
    pub active: bool,
    pub started_ms: u64,
    pub prev_scheme: Option<String>,
    pub created_scheme: Option<String>,
    pub closed: Vec<ClosedApp>,
    pub priority_on: bool,
    pub priority_names: Vec<String>,
    pub messages: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Candidate {
    pub name: String,
    pub display: String,
    pub category: String,
    pub count: usize,
    pub default_on: bool,
}

fn state_path() -> PathBuf {
    paths::data_dir().join("boost.json")
}

pub fn status() -> Result<BoostState, String> {
    Ok(load())
}

fn load() -> BoostState {
    std::fs::read_to_string(state_path()).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
}

fn save(s: &BoostState) {
    if paths::ensure(&paths::data_dir()).is_ok() {
        let _ = std::fs::write(state_path(), serde_json::to_string_pretty(s).unwrap_or_default());
    }
}

fn is_guid(t: &str) -> bool {
    t.len() == 36
        && t.char_indices().all(|(i, c)| if matches!(i, 8 | 13 | 18 | 23) { c == '-' } else { c.is_ascii_hexdigit() })
}

pub fn find_guids(s: &str) -> Vec<String> {
    s.split_whitespace().map(|t| t.trim_matches(|c: char| !c.is_ascii_hexdigit() && c != '-')).filter(|t| is_guid(t)).map(|t| t.to_lowercase()).collect()
}

fn process_counts() -> HashMap<String, usize> {
    let mut sys = System::new();
    sys.refresh_processes();
    let mut m = HashMap::new();
    for p in sys.processes().values() {
        *m.entry(p.name().to_lowercase()).or_insert(0) += 1;
    }
    m
}

/// Running programs that Boost may close.
pub fn candidates() -> Result<Vec<Candidate>, String> {
    let running = process_counts();
    let custom = crate::settings::load().custom_close_apps;
    let mut out: Vec<Candidate> = Vec::new();
    for (name, display, cat, on) in KNOWN {
        if let Some(c) = running.get(*name) {
            out.push(Candidate { name: name.to_string(), display: display.to_string(), category: cat.to_string(), count: *c, default_on: *on });
        }
    }
    for n in custom {
        let l = n.to_lowercase();
        if is_protected(&l) || out.iter().any(|c| c.name == l) {
            continue;
        }
        if let Some(c) = running.get(&l) {
            out.push(Candidate { name: l, display: n, category: "Your list".into(), count: *c, default_on: false });
        }
    }
    Ok(out)
}

fn emulator_process_names() -> Vec<String> {
    scan::load_defs().into_iter().flat_map(|d| d.processes).collect()
}

fn running_emulator_names() -> Vec<String> {
    let running = crate::sys::running_process_names();
    emulator_process_names().into_iter().filter(|p| running.contains(&p.to_lowercase())).collect()
}

fn set_priority(names: &[String], class: &str) {
    let list: Vec<String> = names
        .iter()
        .filter(|n| util::safe_proc_name(n))
        .map(|n| format!("'{}'", n.trim_end_matches(".exe").trim_end_matches(".EXE")))
        .collect();
    if list.is_empty() || !cfg!(windows) {
        return;
    }
    let script = format!(
        "Get-Process -Name {} -ErrorAction SilentlyContinue | ForEach-Object {{ try {{ $_.PriorityClass = '{}' }} catch {{}} }}",
        list.join(","),
        class
    );
    let _ = util::powershell(&script);
}

fn high_performance_plan() -> Result<(Option<String>, Option<String>), String> {
    if !cfg!(windows) {
        return Err("only available on Windows".into());
    }
    let active = util::run("powercfg", &["/getactivescheme"])?;
    let prev = find_guids(&active).into_iter().next();
    let all = find_guids(&util::run("powercfg", &["/list"])?);
    let mut created = None;
    let target = if all.iter().any(|g| g == HIGH_PERFORMANCE) {
        HIGH_PERFORMANCE.to_string()
    } else {
        let out = util::run("powercfg", &["-duplicatescheme", HIGH_PERFORMANCE])?;
        let g = find_guids(&out).into_iter().next().ok_or("could not create a High Performance plan")?;
        created = Some(g.clone());
        g
    };
    if prev.as_deref() == Some(target.as_str()) {
        return Ok((None, created));
    }
    util::run("powercfg", &["/setactive", &target])?;
    Ok((prev, created))
}

fn close_apps(names: &[String]) -> Vec<ClosedApp> {
    let want: HashSet<String> = names.iter().map(|n| n.to_lowercase()).filter(|n| !is_protected(n)).collect();
    let mut sys = System::new();
    sys.refresh_processes();
    let mut closed: Vec<ClosedApp> = Vec::new();
    for p in sys.processes().values() {
        let n = p.name().to_lowercase();
        if !want.contains(&n) {
            continue;
        }
        let path = p.exe().map(|x| x.to_string_lossy().into_owned()).unwrap_or_default();
        if p.kill() && !path.is_empty() && !closed.iter().any(|c| c.name == n) {
            closed.push(ClosedApp { name: n, path });
        }
    }
    closed
}

pub fn start(power_plan: bool, priority: bool, close: Vec<String>) -> Result<BoostState, String> {
    if load().active {
        return Err("Boost is already on.".into());
    }
    let mut st = BoostState { active: true, started_ms: paths::now_ms(), priority_on: priority, ..Default::default() };
    if power_plan {
        match high_performance_plan() {
            Ok((prev, created)) => {
                st.prev_scheme = prev;
                st.created_scheme = created;
                st.messages.push("High Performance power plan is on.".into());
            }
            Err(e) => st.messages.push(format!("Power plan not changed: {}", e)),
        }
    }
    if !close.is_empty() {
        st.closed = close_apps(&close);
        st.messages.push(format!("Closed {} background program(s).", st.closed.len()));
    }
    if priority {
        let names = running_emulator_names();
        set_priority(&names, "AboveNormal");
        st.messages.push(if names.is_empty() {
            "Emulator priority will be raised as soon as an emulator is running.".into()
        } else {
            "Emulator priority raised.".to_string()
        });
        st.priority_names = names;
    }
    save(&st);
    history::add("boost", "Boost started", 0);
    Ok(st)
}

/// Called every few seconds while Boost is on: raises priority of emulators started later.
pub fn tick() -> Result<BoostState, String> {
    let mut st = load();
    if st.active && st.priority_on {
        let now = running_emulator_names();
        let new: Vec<String> = now.iter().filter(|n| !st.priority_names.contains(n)).cloned().collect();
        if !new.is_empty() {
            set_priority(&new, "AboveNormal");
            st.priority_names.extend(new);
            save(&st);
        }
    }
    Ok(st)
}

pub fn stop() -> Result<BoostState, String> {
    let mut st = load();
    if !st.active {
        return Ok(st);
    }
    let mut msgs: Vec<String> = Vec::new();
    if let Some(prev) = st.prev_scheme.clone() {
        match util::run("powercfg", &["/setactive", &prev]) {
            Ok(_) => msgs.push("Previous power plan restored.".into()),
            Err(e) => msgs.push(format!("Could not restore power plan: {}", e)),
        }
    }
    if let Some(created) = st.created_scheme.clone() {
        let _ = util::run("powercfg", &["/delete", &created]);
    }
    let mut reopened = 0;
    for app in &st.closed {
        if std::path::Path::new(&app.path).exists() && std::process::Command::new(&app.path).spawn().is_ok() {
            reopened += 1;
        }
    }
    if !st.closed.is_empty() {
        msgs.push(format!("Reopened {} of {} program(s).", reopened, st.closed.len()));
    }
    if st.priority_on {
        let running = running_emulator_names();
        set_priority(&running, "Normal");
    }
    history::add("boost", "Boost stopped", 0);
    st = BoostState { messages: msgs, ..Default::default() };
    save(&st);
    Ok(st)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guid_parsing() {
        let out = "Power Scheme GUID: 381b4222-f694-41f0-9685-ff5bb260df2e  (Balanced)";
        assert_eq!(find_guids(out), vec!["381b4222-f694-41f0-9685-ff5bb260df2e".to_string()]);
        assert!(find_guids("nothing here").is_empty());
    }

    #[test]
    fn protected_list() {
        for n in ["explorer.exe", "MsMpEng.exe", "Discord.exe", "obs64.exe", "HD-Player.exe", "RtkAudUService64.exe", "emucleaner.exe"] {
            assert!(is_protected(n), "{} must be protected", n);
        }
        assert!(!is_protected("onedrive.exe"));
    }

    #[test]
    fn known_apps_are_not_protected() {
        for (n, _, _, _) in KNOWN {
            assert!(!is_protected(n), "{} is in the closable list but protected", n);
        }
    }
}
