//! Tauri-free state and command logic, so it can be unit-tested without a window.
use luma_engine::{LightSpec, Mode, Show};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, TryRecvError};

pub type Discovery = Receiver<Result<Vec<LightSpec>, String>>;

#[derive(Default)]
pub struct AppState {
    pub show: Show,
    /// Known lights, in discovery order.
    pub known: Vec<LightSpec>,
    /// IPs of the lights the user has ticked.
    pub selected: BTreeSet<String>,
    pub discovery: Option<Discovery>,
    /// Last discovery result or error, shown when the show is idle.
    pub notice: String,
    /// The subnet being scanned, detected from the local network.
    pub subnet: String,
    /// IPs of lights found in an earlier run; their subnets are scanned too.
    pub remembered: Vec<String>,
    /// Subnets the user added by hand (e.g. an IoT network the PC can route to), as "a.b.c".
    pub custom_subnets: Vec<String>,
    /// Where the saved settings are kept, when the app has a data folder.
    pub store_path: Option<PathBuf>,
}

/// What is saved between runs.
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Saved {
    pub lights: Vec<String>,
    pub subnets: Vec<String>,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct LightView {
    pub ip: String,
    pub sku: String,
    pub razer: bool,
    pub segments: usize,
    pub selected: bool,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct StateView {
    pub running: bool,
    pub scanning: bool,
    pub subnet: String,
    pub subnets: Vec<String>,
    pub mode: String,
    pub status: String,
    pub preview: Vec<[u8; 3]>,
    pub lights: Vec<LightView>,
}

/// Parses a mode name from the UI; unknown names are a 400-style error.
pub fn parse_mode(name: &str) -> Result<Mode, String> {
    Mode::from_name(name).ok_or_else(|| format!("400: unknown mode '{name}'"))
}

pub fn set_mode(state: &AppState, name: &str) -> Result<(), String> {
    let mode = parse_mode(name)?;
    *state.show.mode.lock().map_err(|e| e.to_string())? = mode;
    Ok(())
}

/// Keeps only the IPs that are known lights.
pub fn set_selected(state: &mut AppState, ips: &[String]) {
    state.selected = ips.iter().filter(|ip| state.known.iter().any(|l| &l.ip == *ip)).cloned().collect();
}

/// Adds newly found lights (selected by default) and refreshes known ones.
pub fn merge_found(state: &mut AppState, found: Vec<LightSpec>) {
    for spec in found {
        match state.known.iter_mut().find(|l| l.ip == spec.ip) {
            Some(known) => *known = spec,
            None => {
                state.selected.insert(spec.ip.clone());
                state.known.push(spec);
            }
        }
    }
}

/// Picks up a finished discovery, if any. Returns true when it finished successfully.
pub fn poll_discovery(state: &mut AppState) -> bool {
    let Some(rx) = &state.discovery else { return false };
    let result = match rx.try_recv() {
        Ok(result) => result,
        Err(TryRecvError::Empty) => return false,
        Err(TryRecvError::Disconnected) => Err("Discovery stopped unexpectedly".to_string()),
    };
    state.discovery = None;
    match result {
        Ok(found) => {
            state.notice = format!("Found {} light(s)", found.len());
            merge_found(state, found);
            true
        }
        Err(e) => {
            state.notice = e;
            false
        }
    }
}

/// The /24 prefix of an IPv4 address string ("192.168.10.7" -> "192.168.10").
pub fn prefix_of_ip(ip: &str) -> Option<String> {
    let parts: Vec<&str> = ip.split('.').collect();
    (parts.len() == 4).then(|| parts[..3].join("."))
}

/// Loads the saved settings. A missing or unreadable file means none.
pub fn load_saved(path: &Path) -> Saved {
    fs::read_to_string(path).ok().and_then(|text| serde_json::from_str(&text).ok()).unwrap_or_default()
}

/// Saves the known light IPs and the custom subnets so the next run can scan them.
pub fn save_state(state: &AppState) {
    let Some(path) = &state.store_path else { return };
    let saved = Saved {
        lights: state.known.iter().map(|l| l.ip.clone()).collect(),
        subnets: state.custom_subnets.clone(),
    };
    if let Some(dir) = path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    if let Ok(text) = serde_json::to_string_pretty(&saved) {
        let _ = fs::write(path, text);
    }
}

/// Accepts "192.168.10", "192.168.10.0/24" or any address in that /24, and returns "192.168.10".
pub fn normalize_subnet(input: &str) -> Result<String, String> {
    let err = || format!("'{}' is not a subnet. Enter the first three numbers, like 192.168.10", input.trim());
    let head = input.trim().split('/').next().unwrap_or("");
    let parts: Vec<&str> = head.split('.').collect();
    let octets = match parts.len() {
        3 => &parts[..],
        4 => &parts[..3],
        _ => return Err(err()),
    };
    if octets.iter().all(|p| p.parse::<u8>().is_ok()) {
        Ok(octets.join("."))
    } else {
        Err(err())
    }
}

/// Adds a subnet typed by the user; it is scanned on the next scan and saved.
pub fn add_subnet(state: &mut AppState, input: &str) -> Result<(), String> {
    let prefix = normalize_subnet(input)?;
    if !state.custom_subnets.contains(&prefix) {
        state.custom_subnets.push(prefix);
    }
    Ok(())
}

pub fn remove_subnet(state: &mut AppState, prefix: &str) {
    state.custom_subnets.retain(|p| p != prefix);
}

/// Detects the local subnets and scans them for lights, unless a scan is already running.
pub fn begin_discovery(state: &mut AppState) -> Result<(), String> {
    if state.discovery.is_some() {
        return Ok(());
    }
    let mut extra: Vec<String> = state.custom_subnets.clone();
    extra.extend(state.remembered.iter().filter_map(|ip| prefix_of_ip(ip)));
    extra.extend(state.known.iter().filter_map(|l| prefix_of_ip(&l.ip)));
    let prefixes = luma_engine::local_prefixes(&extra);
    if prefixes.is_empty() {
        return Err("No network found. Connect to Wi-Fi or Ethernet and try again.".into());
    }
    state.subnet = prefixes.iter().map(|p| format!("{p}.0/24")).collect::<Vec<_>>().join(", ");
    state.discovery = Some(luma_engine::find_lights(prefixes));
    Ok(())
}

/// Starts the show on the selected lights.
pub fn start(state: &AppState) -> Result<(), String> {
    if state.show.is_running() {
        return Err("The show is already running".into());
    }
    let specs = state.known.iter().filter(|l| state.selected.contains(&l.ip)).cloned().collect();
    state.show.start(specs)
}

pub fn stop(state: &AppState) {
    state.show.stop();
}

/// The state the UI polls; call `poll_discovery` first.
pub fn view(state: &AppState) -> StateView {
    let running = state.show.is_running();
    let status = if running {
        state.show.status.lock().map(|s| s.clone()).unwrap_or_default()
    } else if state.discovery.is_some() {
        "Searching for lights...".to_string()
    } else if !state.notice.is_empty() {
        state.notice.clone()
    } else {
        state.show.status.lock().map(|s| s.clone()).unwrap_or_default()
    };
    let mode = state.show.mode.lock().map(|m| m.label()).unwrap_or_default();
    let preview = state.show.preview.lock().map(|p| p.iter().map(|&(r, g, b)| [r, g, b]).collect()).unwrap_or_default();
    let lights = state
        .known
        .iter()
        .map(|l| LightView {
            ip: l.ip.clone(),
            sku: l.sku.clone(),
            razer: l.razer,
            segments: l.segments,
            selected: state.selected.contains(&l.ip),
        })
        .collect();
    StateView {
        running,
        scanning: state.discovery.is_some(),
        subnet: state.subnet.clone(),
        subnets: state.custom_subnets.clone(),
        mode,
        status,
        preview,
        lights,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(ip: &str, sku: &str) -> LightSpec {
        let razer = sku == "H6022";
        LightSpec { sku: sku.into(), ip: ip.into(), razer, segments: if razer { 15 } else { 1 } }
    }

    fn with_lights() -> AppState {
        let mut s = AppState::default();
        merge_found(&mut s, vec![spec("10.0.0.1", "H6022"), spec("10.0.0.2", "H6000")]);
        s
    }

    #[test]
    fn mode_parsing_accepts_ui_names() {
        assert_eq!(parse_mode("microphone"), Ok(Mode::Mic));
        assert_eq!(parse_mode("screen"), Ok(Mode::Screen));
        assert_eq!(parse_mode("chase"), Ok(Mode::Pattern("chase")));
        assert_eq!(parse_mode("Rainbow"), Ok(Mode::Pattern("rainbow")));
    }

    #[test]
    fn unknown_mode_is_a_400_style_error() {
        let err = parse_mode("disco").unwrap_err();
        assert!(err.starts_with("400"), "{err}");
        assert!(err.contains("disco"));
    }

    #[test]
    fn set_mode_updates_the_show() {
        let s = AppState::default();
        set_mode(&s, "screen").unwrap();
        assert_eq!(*s.show.mode.lock().unwrap(), Mode::Screen);
        assert!(set_mode(&s, "disco").is_err());
        assert_eq!(*s.show.mode.lock().unwrap(), Mode::Screen);
    }

    #[test]
    fn found_lights_are_selected_by_default() {
        let s = with_lights();
        assert_eq!(s.known.len(), 2);
        assert_eq!(s.selected.len(), 2);
    }

    #[test]
    fn merging_keeps_user_selection_and_skips_duplicates() {
        let mut s = with_lights();
        set_selected(&mut s, &["10.0.0.1".into()]);
        merge_found(&mut s, vec![spec("10.0.0.1", "H6022"), spec("10.0.0.3", "H6000")]);
        assert_eq!(s.known.len(), 3);
        assert!(s.selected.contains("10.0.0.1"));
        assert!(!s.selected.contains("10.0.0.2"));
        assert!(s.selected.contains("10.0.0.3"));
    }

    #[test]
    fn selection_ignores_unknown_ips() {
        let mut s = with_lights();
        set_selected(&mut s, &["10.0.0.2".into(), "9.9.9.9".into()]);
        assert_eq!(s.selected.iter().collect::<Vec<_>>(), vec!["10.0.0.2"]);
    }

    #[test]
    fn view_marks_selected_lights() {
        let mut s = with_lights();
        set_selected(&mut s, &["10.0.0.2".into()]);
        let v = view(&s);
        assert!(!v.running);
        assert_eq!(v.mode, "rainbow");
        assert_eq!(v.lights[0], LightView { ip: "10.0.0.1".into(), sku: "H6022".into(), razer: true, segments: 15, selected: false });
        assert!(v.lights[1].selected);
    }

    #[test]
    fn view_status_reports_searching_then_notice() {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut s = AppState { discovery: Some(rx), ..Default::default() };
        assert_eq!(view(&s).status, "Searching for lights...");
        tx.send(Err("no lights".into())).unwrap();
        poll_discovery(&mut s);
        assert!(s.discovery.is_none());
        assert_eq!(view(&s).status, "no lights");
    }

    #[test]
    fn start_needs_a_selected_light() {
        let mut s = with_lights();
        set_selected(&mut s, &[]);
        assert_eq!(start(&s), Err("Select at least one light".to_string()));
    }

    #[test]
    fn subnet_input_is_normalized() {
        assert_eq!(normalize_subnet("192.168.10"), Ok("192.168.10".into()));
        assert_eq!(normalize_subnet(" 192.168.10.0/24 "), Ok("192.168.10".into()));
        assert_eq!(normalize_subnet("10.0.5.77"), Ok("10.0.5".into()));
        assert!(normalize_subnet("192.168").is_err());
        assert!(normalize_subnet("192.168.300").is_err());
        assert!(normalize_subnet("lights").is_err());
    }

    #[test]
    fn added_subnets_are_kept_once() {
        let mut s = AppState::default();
        add_subnet(&mut s, "192.168.10.0/24").unwrap();
        add_subnet(&mut s, "192.168.10").unwrap();
        assert_eq!(s.custom_subnets, vec!["192.168.10"]);
        remove_subnet(&mut s, "192.168.10");
        assert!(s.custom_subnets.is_empty());
    }

    #[test]
    fn remembered_lights_give_their_subnet() {
        assert_eq!(prefix_of_ip("192.168.10.42"), Some("192.168.10".into()));
        assert_eq!(prefix_of_ip("not-an-ip"), None);
    }

    #[test]
    fn begin_discovery_keeps_a_running_scan() {
        let (_tx, rx) = std::sync::mpsc::channel();
        let mut s = AppState { discovery: Some(rx), subnet: "10.1.1.0/24".into(), ..Default::default() };
        assert!(begin_discovery(&mut s).is_ok());
        assert_eq!(s.subnet, "10.1.1.0/24");
        assert!(view(&s).scanning);
    }
}
