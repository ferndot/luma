//! Tauri-free state and command logic, so it can be unit-tested without a window.
use luma_core::effects::{Effect, SequenceStep, builtin_effects};
use luma_engine::{LightSpec, Placed, Sequence, Show, Step};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub type Discovery = Receiver<Result<Vec<LightSpec>, String>>;
pub type NameFetch = Receiver<Result<BTreeMap<String, String>, String>>;

const DEVICES_URL: &str = "https://openapi.api.govee.com/router/api/v1/user/devices";

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
    /// Names from the Govee cloud account, keyed by device id.
    pub cloud_names: BTreeMap<String, String>,
    /// Named sets of lights, by device id.
    pub groups: Vec<Group>,
    /// The light being pressed, if any, with the state every group member had before.
    pub session: Option<Session>,
    /// Govee cloud API key, used only to read device names.
    pub api_key: Option<String>,
    pub names_fetch: Option<NameFetch>,
    /// Result of the last name fetch, shown in the settings.
    pub key_status: String,
    /// Where the saved settings are kept, when the app has a data folder.
    pub store_path: Option<PathBuf>,
    /// The effect library: built-in effects, the user's own, and sequences of them.
    pub effects: Vec<NamedEffect>,
    /// The effect currently playing, if one was started.
    pub active_effect: Option<String>,
}

/// An effect in the library, with the id the sidebar and sequences refer to it by.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct NamedEffect {
    pub id: String,
    pub name: String,
    pub effect: Effect,
}

/// What is saved between runs.
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Saved {
    pub lights: Vec<String>,
    pub subnets: Vec<String>,
    pub api_key: Option<String>,
    pub groups: Vec<Group>,
    pub effects: Vec<NamedEffect>,
}

/// A named set of lights, stored by device id so it survives IP changes.
/// `positions` place each member in the room: x and y from 0 (left/top) to 1 (right/bottom).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Group {
    pub id: String,
    pub name: String,
    pub members: Vec<String>,
    #[serde(default)]
    pub positions: BTreeMap<String, [f32; 2]>,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct GroupView {
    pub id: String,
    pub name: String,
    pub members: Vec<String>,
    pub positions: BTreeMap<String, [f32; 2]>,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct LightView {
    pub id: String,
    pub name: String,
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
    pub has_key: bool,
    pub key_status: String,
    pub groups: Vec<GroupView>,
    pub effects: Vec<NamedEffect>,
    /// Id of the effect playing, if one is.
    pub active_effect: Option<String>,
    pub status: String,
    pub preview: Vec<[u8; 3]>,
    /// Room spot of each preview segment, in the same order.
    pub positions: Vec<[f32; 2]>,
    pub lights: Vec<LightView>,
}

/// The effect library starts with the built-in effects, plus any the saved list is missing.
pub fn with_builtins(mut saved: Vec<NamedEffect>) -> Vec<NamedEffect> {
    for (id, name, effect) in builtin_effects() {
        if !saved.iter().any(|e| e.id == id) {
            saved.push(NamedEffect { id: id.into(), name: name.into(), effect });
        }
    }
    saved
}

fn builtin_ids() -> Vec<&'static str> {
    builtin_effects().into_iter().map(|(id, _, _)| id).collect()
}

fn new_id(prefix: char) -> String {
    format!("{prefix}{}", SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0))
}

/// Saves an effect (new when its id is empty) and returns its id. Built-in effects can be edited too.
pub fn save_effect(state: &mut AppState, mut effect: NamedEffect) -> Result<String, String> {
    effect.name = effect.name.trim().to_string();
    if effect.name.is_empty() || effect.name.chars().count() > 40 {
        return Err("Effect names need 1 to 40 characters".into());
    }
    effect.effect.check()?;
    check_steps(&effect.effect, &state.effects)?;
    if state.effects.iter().any(|e| e.id != effect.id && e.name.eq_ignore_ascii_case(&effect.name)) {
        return Err(format!("There is already an effect named '{}'", effect.name));
    }
    if effect.id.is_empty() {
        effect.id = new_id('e');
    }
    match state.effects.iter_mut().find(|e| e.id == effect.id) {
        Some(existing) => *existing = effect.clone(),
        None => state.effects.push(effect.clone()),
    }
    Ok(effect.id)
}

/// Puts a built-in effect back to how it was installed.
pub fn reset_effect(state: &mut AppState, id: &str) -> Result<(), String> {
    let (_, name, effect) = builtin_effects()
        .into_iter()
        .find(|(b, _, _)| *b == id)
        .ok_or("Only built-in effects can be reset")?;
    match state.effects.iter_mut().find(|e| e.id == id) {
        Some(existing) => {
            existing.name = name.into();
            existing.effect = effect;
        }
        None => state.effects.push(NamedEffect { id: id.into(), name: name.into(), effect }),
    }
    Ok(())
}

/// Deletes one of the user's own effects. Sequences that still use it report that when played.
pub fn delete_effect(state: &mut AppState, id: &str) -> Result<(), String> {
    if builtin_ids().contains(&id) {
        return Err("Built-in effects can be reset, not deleted".into());
    }
    state.effects.retain(|e| e.id != id);
    Ok(())
}

/// A sequence's steps must each name a library effect, and a sequence cannot hold another sequence.
fn check_steps(effect: &Effect, library: &[NamedEffect]) -> Result<(), String> {
    let Effect::Sequence { steps, .. } = effect else { return Ok(()) };
    for step in steps {
        match library.iter().find(|e| e.id == step.effect) {
            None => return Err("A step uses an effect that is not in the library".into()),
            Some(e) if matches!(e.effect, Effect::Sequence { .. }) => return Err("A sequence cannot use another sequence".into()),
            Some(_) => {}
        }
    }
    Ok(())
}

fn effect_named(state: &AppState, id: &str) -> Result<Effect, String> {
    state.effects.iter().find(|e| e.id == id).map(|e| e.effect.clone()).ok_or_else(|| "That effect no longer exists".to_string())
}

/// The show's sequence for an effect. A sequence's steps are looked up now, so edits to them apply;
/// any other effect plays as a one-step sequence that repeats.
fn resolve(state: &AppState, id: &str) -> Result<Sequence, String> {
    match effect_named(state, id)? {
        Effect::Sequence { steps, repeat } => {
            let steps = steps
                .iter()
                .map(|s: &SequenceStep| effect_named(state, &s.effect).map(|effect| Step { effect, secs: s.secs }))
                .collect::<Result<Vec<_>, _>>()?;
            if steps.iter().any(|s| matches!(s.effect, Effect::Sequence { .. })) {
                return Err("A sequence cannot use another sequence".into());
            }
            Ok(Sequence { steps, repeat })
        }
        effect => Ok(Sequence { steps: vec![Step { effect, secs: 1.0 }], repeat: true }),
    }
}

/// Plays an effect on the show. A sequence plays its steps in order; anything else repeats until another is picked.
pub fn play_effect(state: &mut AppState, id: &str) -> Result<(), String> {
    let sequence = resolve(state, id)?;
    *state.show.sequence.lock().map_err(|e| e.to_string())? = Some(sequence);
    state.active_effect = Some(id.into());
    Ok(())
}

/// One preview to draw: an effect on a grid of `cols` by `rows` room spots.
#[derive(Deserialize)]
pub struct PreviewRequest {
    pub effect: Effect,
    pub cols: usize,
    pub rows: usize,
}

/// Colors of a grid of room spots (`cols` by `rows`, row by row) at time `t`, for previews and icons.
/// Live sources have no color of their own, so they show black.
pub fn preview(effect: &Effect, t: f64, cols: usize, rows: usize) -> Vec<[u8; 3]> {
    let (cols, rows) = (cols.clamp(1, 64), rows.clamp(1, 64));
    (0..rows)
        .flat_map(|r| {
            (0..cols).map(move |c| {
                let x = (c as f32 + 0.5) / cols as f32;
                let y = (r as f32 + 0.5) / rows as f32;
                let (red, green, blue) = effect.color_at(x, y, t).unwrap_or((0, 0, 0));
                [red, green, blue]
            })
        })
        .collect()
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
        api_key: state.api_key.clone(),
        groups: state.groups.clone(),
        effects: state.effects.clone(),
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

/// The name shown for a light when the user has not set one: its model and last IP number.
pub fn default_name(spec: &LightSpec) -> String {
    let last = spec.ip.rsplit('.').next().unwrap_or(&spec.ip);
    format!("{} · .{last}", spec.sku)
}

fn clean_group_name(name: &str) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Give the group a name".into());
    }
    if name.chars().count() > 40 {
        return Err("Group names can be at most 40 characters".into());
    }
    Ok(name.to_string())
}

fn find_group<'a>(state: &'a mut AppState, id: &str) -> Result<&'a mut Group, String> {
    state.groups.iter_mut().find(|g| g.id == id).ok_or_else(|| "That group no longer exists".to_string())
}

/// Makes an empty group and returns its id.
pub fn create_group(state: &mut AppState, name: &str) -> Result<String, String> {
    let name = clean_group_name(name)?;
    if state.groups.iter().any(|g| g.name.eq_ignore_ascii_case(&name)) {
        return Err(format!("There is already a group named '{name}'"));
    }
    let id = format!("g{}", SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0));
    state.groups.push(Group { id: id.clone(), name, members: Vec::new(), positions: BTreeMap::new() });
    Ok(id)
}

pub fn rename_group(state: &mut AppState, id: &str, name: &str) -> Result<(), String> {
    let name = clean_group_name(name)?;
    if state.groups.iter().any(|g| g.id != id && g.name.eq_ignore_ascii_case(&name)) {
        return Err(format!("There is already a group named '{name}'"));
    }
    find_group(state, id)?.name = name;
    Ok(())
}

/// Sets which devices are in a group. Only known devices are kept; positions of removed devices are dropped.
pub fn set_group_members(state: &mut AppState, id: &str, members: &[String]) -> Result<(), String> {
    let known: BTreeSet<&str> = state.known.iter().map(|l| l.id.as_str()).collect();
    let members: Vec<String> = members.iter().filter(|m| known.contains(m.as_str())).cloned().collect();
    let group = find_group(state, id)?;
    group.positions.retain(|device, _| members.contains(device));
    group.members = members;
    Ok(())
}

/// Places a member of the group. Coordinates are clamped to the room, 0 to 1.
pub fn set_group_position(state: &mut AppState, id: &str, device: &str, x: f32, y: f32) -> Result<(), String> {
    let group = find_group(state, id)?;
    if !group.members.iter().any(|m| m == device) {
        return Err("That device is not in the group".into());
    }
    group.positions.insert(device.to_string(), [x.clamp(0.0, 1.0), y.clamp(0.0, 1.0)]);
    Ok(())
}

/// The group's known lights with the room spot each one sits at, in the order a scene runs them:
/// left to right, then top to bottom. Lights without a spot go last, spread along the bottom.
pub fn scene_order(state: &mut AppState, id: &str) -> Result<Vec<Placed>, String> {
    let group = find_group(state, id)?.clone();
    let mut lights: Vec<(Option<(f32, f32)>, LightSpec)> = state
        .known
        .iter()
        .filter(|l| group.members.contains(&l.id))
        .map(|l| (group.positions.get(&l.id).map(|p| (p[0], p[1])), l.clone()))
        .collect();
    if lights.is_empty() {
        return Err("Add lights to the group first".into());
    }
    lights.sort_by(|a, b| match (a.0, b.0) {
        (Some(p), Some(q)) => p.0.total_cmp(&q.0).then(p.1.total_cmp(&q.1)),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    });
    let unplaced = lights.len() as f32;
    Ok(lights
        .into_iter()
        .enumerate()
        .map(|(i, (at, spec))| Placed { spec, at: at.unwrap_or(((i as f32 + 0.5) / unplaced, 0.5)) })
        .collect())
}

/// Starts the show on the group's lights in room order, with the current mode.
pub fn start_group(state: &mut AppState, id: &str) -> Result<(), String> {
    if state.show.is_running() {
        return Err("The show is already running".into());
    }
    let placed = scene_order(state, id)?;
    state.show.start(placed)
}

/// A press on a light. Holds the pressed light on its color and remembers how every group member was,
/// so release can put them all back.
pub struct Session {
    pub id: String,
    pub members: Vec<(LightSpec, Option<(bool, (u8, u8, u8))>)>,
    pub lit: luma_engine::Lit,
}

/// Ends the current press: stops the lit light and restores every member to its prior state.
fn end_session(state: &mut AppState) {
    if let Some(session) = state.session.take() {
        session.lit.end();
        // All members at once: each one waits on its own razer settle time and network reply.
        std::thread::scope(|s| {
            for (spec, prior) in &session.members {
                s.spawn(move || luma_engine::restore(spec, *prior));
            }
        });
    }
}

/// Lights a device in a solid color while pressed. The rest of its group goes off.
/// Not while a show is running.
pub fn light_device(state: &mut AppState, group: &str, id: &str, color: [u8; 3]) -> Result<(), String> {
    if state.show.is_running() {
        return Err("Stop the scene to light lights".into());
    }
    let member_ids = find_group(state, group)?.members.clone();
    if !member_ids.iter().any(|m| m == id) {
        return Err("That light is not in the group".into());
    }
    let specs: Vec<LightSpec> = state.known.iter().filter(|l| member_ids.contains(&l.id)).cloned().collect();
    let lit_spec = specs.iter().find(|l| l.id == id).ok_or("That light is not known")?.clone();

    // Finish the previous press first so the states read below are the real ones.
    end_session(state);
    let ips: Vec<String> = specs.iter().map(|l| l.ip.clone()).collect();
    let priors = luma_engine::statuses(&ips, Duration::from_millis(400)).unwrap_or_default();
    let members: Vec<(LightSpec, Option<(bool, (u8, u8, u8))>)> =
        specs.into_iter().map(|l| {
            let prior = priors.get(&l.ip).copied();
            (l, prior)
        }).collect();
    std::thread::scope(|s| {
        for (spec, _) in members.iter().filter(|(l, _)| l.id != id) {
            s.spawn(move || luma_engine::off(spec));
        }
    });
    let lit = luma_engine::light_up(lit_spec, (color[0], color[1], color[2]));
    state.session = Some(Session { id: id.to_string(), members, lit });
    Ok(())
}

/// Ends the press on this device, if it is the one being pressed.
pub fn release_device(state: &mut AppState, id: &str) {
    if state.session.as_ref().is_some_and(|s| s.id == id) {
        end_session(state);
    }
}

pub fn delete_group(state: &mut AppState, id: &str) {
    state.groups.retain(|g| g.id != id);
}

/// Stores the Govee API key (empty clears it) and fetches device names with it.
pub fn set_api_key(state: &mut AppState, key: &str) {
    let key = key.trim();
    state.api_key = (!key.is_empty()).then(|| key.to_string());
    state.cloud_names.clear();
    state.key_status.clear();
    begin_name_fetch(state);
}

/// Fetches device names from the Govee cloud in the background, if a key is set.
pub fn begin_name_fetch(state: &mut AppState) {
    let Some(key) = state.api_key.clone() else { return };
    if state.names_fetch.is_some() {
        return;
    }
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(fetch_cloud_names(&key));
    });
    state.names_fetch = Some(rx);
    state.key_status = "Loading names from Govee…".into();
}

/// Picks up a finished name fetch. Returns true when names arrived.
pub fn poll_names(state: &mut AppState) -> bool {
    let Some(rx) = &state.names_fetch else { return false };
    let result = match rx.try_recv() {
        Ok(result) => result,
        Err(TryRecvError::Empty) => return false,
        Err(TryRecvError::Disconnected) => Err("Name fetch stopped unexpectedly".to_string()),
    };
    state.names_fetch = None;
    match result {
        Ok(names) => {
            state.key_status = format!("Named {} device(s) from Govee", names.len());
            state.cloud_names = names;
            true
        }
        Err(e) => {
            state.key_status = e;
            false
        }
    }
}

fn fetch_cloud_names(key: &str) -> Result<BTreeMap<String, String>, String> {
    let mut resp = ureq::get(DEVICES_URL)
        .header("Govee-API-Key", key)
        .call()
        .map_err(|e| format!("Could not reach Govee: {e}"))?;
    let body: Value = resp.body_mut().read_json().map_err(|e| format!("Unexpected reply from Govee: {e}"))?;
    parse_device_names(&body)
}

/// Reads `deviceName` for each device in a Govee cloud device-list reply.
pub fn parse_device_names(body: &Value) -> Result<BTreeMap<String, String>, String> {
    let list = body["data"].as_array().ok_or_else(|| "Govee rejected the key or returned no devices".to_string())?;
    Ok(list
        .iter()
        .filter_map(|d| Some((d["device"].as_str()?.to_string(), d["deviceName"].as_str()?.to_string())))
        .collect())
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
    // Lights that were never placed run left to right in the order they were found.
    let specs: Vec<LightSpec> = state.known.iter().filter(|l| state.selected.contains(&l.ip)).cloned().collect();
    let n = specs.len() as f32;
    let placed = specs
        .into_iter()
        .enumerate()
        .map(|(i, spec)| Placed { spec, at: ((i as f32 + 0.5) / n, 0.5) })
        .collect();
    state.show.start(placed)
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
    let preview = state.show.preview.lock().map(|p| p.iter().map(|&(r, g, b)| [r, g, b]).collect()).unwrap_or_default();
    let positions = state.show.positions.lock().map(|p| p.iter().map(|&(x, y)| [x, y]).collect()).unwrap_or_default();
    let lights = state
        .known
        .iter()
        .map(|l| LightView {
            id: l.id.clone(),
            name: state.cloud_names.get(&l.id).cloned().unwrap_or_else(|| default_name(l)),
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
        has_key: state.api_key.is_some(),
        key_status: state.key_status.clone(),
        groups: state
            .groups
            .iter()
            .map(|g| GroupView {
                id: g.id.clone(),
                name: g.name.clone(),
                members: g.members.clone(),
                positions: g.positions.clone(),
            })
            .collect(),
        effects: state.effects.clone(),
        active_effect: state.active_effect.clone(),
        status,
        preview,
        positions,
        lights,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(ip: &str, sku: &str) -> LightSpec {
        let razer = sku == "H6022";
        let id = format!("id-{ip}");
        LightSpec { id, sku: sku.into(), ip: ip.into(), razer, segments: if razer { 15 } else { 1 } }
    }

    fn with_lights() -> AppState {
        let mut s = AppState::default();
        merge_found(&mut s, vec![spec("10.0.0.1", "H6022"), spec("10.0.0.2", "H6000")]);
        s
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
        assert_eq!(
            v.lights[0],
            LightView {
                id: "id-10.0.0.1".into(),
                name: "H6022 · .1".into(),
                ip: "10.0.0.1".into(),
                sku: "H6022".into(),
                razer: true,
                segments: 15,
                selected: false,
            }
        );
        assert!(v.lights[1].selected);
    }

    #[test]
    fn groups_start_with_no_devices_and_names_are_unique() {
        let mut s = with_lights();
        let id = create_group(&mut s, " Desk ").unwrap();
        assert_eq!(s.groups[0].name, "Desk");
        assert!(s.groups[0].members.is_empty());
        assert!(create_group(&mut s, "desk").is_err());
        assert!(create_group(&mut s, "  ").is_err());
        assert!(rename_group(&mut s, &id, "Desk").is_ok());
    }

    #[test]
    fn scene_needs_members_and_keeps_only_known_ones() {
        let mut s = with_lights();
        let id = create_group(&mut s, "Desk").unwrap();
        assert!(scene_order(&mut s, &id).is_err());
        set_group_members(&mut s, &id, &["id-10.0.0.2".into(), "unknown".into()]).unwrap();
        assert_eq!(s.groups[0].members, vec!["id-10.0.0.2"]);
        let order: Vec<String> = scene_order(&mut s, &id).unwrap().into_iter().map(|p| p.spec.id).collect();
        assert_eq!(order, vec!["id-10.0.0.2"]);
    }

    #[test]
    fn scene_runs_left_to_right_then_top_to_bottom() {
        let mut s = with_lights();
        let id = create_group(&mut s, "Room").unwrap();
        set_group_members(&mut s, &id, &["id-10.0.0.1".into(), "id-10.0.0.2".into()]).unwrap();
        set_group_position(&mut s, &id, "id-10.0.0.1", 0.9, 0.1).unwrap();
        set_group_position(&mut s, &id, "id-10.0.0.2", 0.1, 0.9).unwrap();
        let order: Vec<String> = scene_order(&mut s, &id).unwrap().into_iter().map(|p| p.spec.id).collect();
        assert_eq!(order, vec!["id-10.0.0.2", "id-10.0.0.1"]);
    }

    #[test]
    fn positions_are_kept_for_members_only_and_clamped() {
        let mut s = with_lights();
        let id = create_group(&mut s, "Room").unwrap();
        set_group_members(&mut s, &id, &["id-10.0.0.1".into(), "id-10.0.0.2".into()]).unwrap();
        set_group_position(&mut s, &id, "id-10.0.0.1", 1.5, -2.0).unwrap();
        assert_eq!(s.groups[0].positions["id-10.0.0.1"], [1.0, 0.0]);
        assert!(set_group_position(&mut s, &id, "id-10.0.0.3", 0.5, 0.5).is_err());
        set_group_members(&mut s, &id, &["id-10.0.0.2".into()]).unwrap();
        assert!(s.groups[0].positions.is_empty());
    }

    #[test]
    fn groups_can_be_deleted_and_appear_in_the_view() {
        let mut s = with_lights();
        let id = create_group(&mut s, "Pair").unwrap();
        set_group_members(&mut s, &id, &["id-10.0.0.1".into()]).unwrap();
        assert_eq!(view(&s).groups[0].members, vec!["id-10.0.0.1"]);
        delete_group(&mut s, &id);
        assert!(view(&s).groups.is_empty());
    }

    #[test]
    fn cloud_names_are_shown_when_present() {
        let mut s = with_lights();
        s.cloud_names.insert("id-10.0.0.1".into(), "Desk strip".into());
        assert_eq!(view(&s).lights[0].name, "Desk strip");
        assert_eq!(view(&s).lights[1].name, "H6000 · .2");
    }

    #[test]
    fn device_names_are_read_from_cloud_reply() {
        let body = serde_json::json!({"data": [
            {"device": "AA:01", "deviceName": "Desk strip"},
            {"device": "AA:02"}
        ]});
        let names = parse_device_names(&body).unwrap();
        assert_eq!(names.len(), 1);
        assert_eq!(names["AA:01"], "Desk strip");
        assert!(parse_device_names(&serde_json::json!({"message": "bad key"})).is_err());
    }

    /// A state with the built-in effects in its library, as on a real start.
    fn with_effects() -> AppState {
        AppState { effects: with_builtins(vec![]), ..Default::default() }
    }

    fn step(effect: &str, secs: f32) -> SequenceStep {
        SequenceStep { effect: effect.into(), secs }
    }

    fn sequence(name: &str, steps: Vec<SequenceStep>) -> NamedEffect {
        NamedEffect { id: String::new(), name: name.into(), effect: Effect::Sequence { steps, repeat: true } }
    }

    #[test]
    fn sequences_are_checked_against_the_library() {
        let mut s = with_effects();
        assert!(save_effect(&mut s, sequence("Bad", vec![step("disco", 5.0)])).is_err());
        assert!(save_effect(&mut s, sequence("Good", vec![step("chase", 5.0)])).is_ok());
    }

    #[test]
    fn sequences_cannot_hold_sequences() {
        let mut s = with_effects();
        let inner = save_effect(&mut s, sequence("Inner", vec![step("chase", 5.0)])).unwrap();
        assert!(save_effect(&mut s, sequence("Outer", vec![step(&inner, 5.0)])).is_err());
    }

    #[test]
    fn playing_a_sequence_resolves_its_steps_when_played() {
        let mut s = with_effects();
        let id = save_effect(&mut s, sequence("Seq", vec![step("chase", 2.0), step("solid", 3.0)])).unwrap();
        play_effect(&mut s, &id).unwrap();
        assert_eq!(view(&s).active_effect, Some(id.clone()));
        // An edit to a step's effect shows up the next time the sequence plays.
        let mut solid = s.effects.iter().find(|e| e.id == "solid").unwrap().clone();
        solid.effect = Effect::Solid { color: (9, 9, 9) };
        save_effect(&mut s, solid).unwrap();
        let played = resolve(&s, &id).unwrap();
        assert_eq!(played.steps[1].effect, Effect::Solid { color: (9, 9, 9) });
        assert_eq!(played.steps[1].secs, 3.0);
        play_effect(&mut s, "screen").unwrap();
        assert_eq!(view(&s).active_effect, Some("screen".to_string()));
    }

    #[test]
    fn built_in_effects_can_be_edited_and_reset_but_not_deleted() {
        let mut s = with_effects();
        let mut rainbow = s.effects.iter().find(|e| e.id == "rainbow").unwrap().clone();
        rainbow.effect = Effect::Solid { color: (1, 2, 3) };
        save_effect(&mut s, rainbow).unwrap();
        assert_eq!(s.effects.iter().find(|e| e.id == "rainbow").unwrap().effect, Effect::Solid { color: (1, 2, 3) });
        reset_effect(&mut s, "rainbow").unwrap();
        assert!(matches!(s.effects.iter().find(|e| e.id == "rainbow").unwrap().effect, Effect::Gradient { .. }));
        assert!(delete_effect(&mut s, "rainbow").is_err());
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
    fn preview_is_a_grid_of_room_spots() {
        let cells = preview(&Effect::Solid { color: (1, 2, 3) }, 0.0, 24, 14);
        assert_eq!(cells.len(), 24 * 14);
        assert!(cells.iter().all(|c| *c == [1, 2, 3]));
        assert_eq!(preview(&Effect::Solid { color: (1, 2, 3) }, 0.0, 8, 4).len(), 32);
        assert!(preview(&Effect::Screen, 0.0, 8, 4).iter().all(|c| *c == [0, 0, 0]));
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
