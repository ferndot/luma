mod logic;

use logic::{AppState, StateView};
use std::sync::{Mutex, MutexGuard};
use tauri::{Manager, State};

type Shared = Mutex<AppState>;

fn lock(app: &Shared) -> Result<MutexGuard<'_, AppState>, String> {
    app.lock().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_state(app: State<'_, Shared>) -> Result<StateView, String> {
    let mut s = lock(&app)?;
    if logic::poll_discovery(&mut s) {
        logic::save_state(&s);
    }
    logic::poll_names(&mut s);
    Ok(logic::view(&s))
}

#[tauri::command]
fn create_group(app: State<'_, Shared>, name: String) -> Result<String, String> {
    let mut s = lock(&app)?;
    let id = logic::create_group(&mut s, &name)?;
    logic::save_state(&s);
    Ok(id)
}

#[tauri::command]
fn rename_group(app: State<'_, Shared>, id: String, name: String) -> Result<(), String> {
    let mut s = lock(&app)?;
    logic::rename_group(&mut s, &id, &name)?;
    logic::save_state(&s);
    Ok(())
}

#[tauri::command]
fn set_group_members(app: State<'_, Shared>, id: String, members: Vec<String>) -> Result<(), String> {
    let mut s = lock(&app)?;
    logic::set_group_members(&mut s, &id, &members)?;
    logic::save_state(&s);
    Ok(())
}

#[tauri::command]
fn set_group_position(app: State<'_, Shared>, id: String, device: String, x: f32, y: f32) -> Result<(), String> {
    let mut s = lock(&app)?;
    logic::set_group_position(&mut s, &id, &device, x, y)?;
    logic::save_state(&s);
    Ok(())
}

#[tauri::command]
fn start_group(app: State<'_, Shared>, id: String) -> Result<(), String> {
    let mut s = lock(&app)?;
    logic::start_group(&mut s, &id)
}

#[tauri::command]
fn light_device(app: State<'_, Shared>, group: String, device: String, color: [u8; 3]) -> Result<(), String> {
    let mut s = lock(&app)?;
    logic::light_device(&mut s, &group, &device, color)
}

#[tauri::command]
fn release_device(app: State<'_, Shared>, device: String) -> Result<(), String> {
    let mut s = lock(&app)?;
    logic::release_device(&mut s, &device);
    Ok(())
}

#[tauri::command]
fn delete_group(app: State<'_, Shared>, id: String) -> Result<(), String> {
    let mut s = lock(&app)?;
    logic::delete_group(&mut s, &id);
    logic::save_state(&s);
    Ok(())
}

#[tauri::command]
fn set_api_key(app: State<'_, Shared>, key: String) -> Result<(), String> {
    let mut s = lock(&app)?;
    logic::set_api_key(&mut s, &key);
    logic::save_state(&s);
    Ok(())
}

#[tauri::command]
fn add_subnet(app: State<'_, Shared>, subnet: String) -> Result<(), String> {
    let mut s = lock(&app)?;
    logic::add_subnet(&mut s, &subnet)?;
    logic::save_state(&s);
    Ok(())
}

#[tauri::command]
fn remove_subnet(app: State<'_, Shared>, subnet: String) -> Result<(), String> {
    let mut s = lock(&app)?;
    logic::remove_subnet(&mut s, &subnet);
    logic::save_state(&s);
    Ok(())
}

#[tauri::command]
fn set_mode(app: State<'_, Shared>, name: String) -> Result<(), String> {
    let s = lock(&app)?;
    logic::set_mode(&s, &name)
}

#[tauri::command]
fn find_lights(app: State<'_, Shared>) -> Result<(), String> {
    let mut s = lock(&app)?;
    logic::begin_discovery(&mut s)
}

#[tauri::command]
fn set_selected(app: State<'_, Shared>, ips: Vec<String>) -> Result<(), String> {
    let mut s = lock(&app)?;
    logic::set_selected(&mut s, &ips);
    Ok(())
}

#[tauri::command]
fn start(app: State<'_, Shared>) -> Result<(), String> {
    let s = lock(&app)?;
    logic::start(&s)
}

#[tauri::command]
fn stop(app: State<'_, Shared>) -> Result<(), String> {
    let s = lock(&app)?;
    logic::stop(&s);
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Lights and added subnets from earlier runs are kept in the app's data folder.
            let store_path = app.path().app_data_dir().ok().map(|dir| dir.join("settings.json"));
            let saved = store_path.as_deref().map(logic::load_saved).unwrap_or_default();
            let mut state = AppState {
                store_path,
                remembered: saved.lights,
                custom_subnets: saved.subnets,
                api_key: saved.api_key,
                groups: saved.groups,
                ..Default::default()
            };
            logic::begin_name_fetch(&mut state);
            app.manage(Mutex::new(state));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_state,
            set_mode,
            find_lights,
            add_subnet,
            remove_subnet,
            set_api_key,
            create_group,
            rename_group,
            set_group_members,
            set_group_position,
            start_group,
            light_device,
            release_device,
            delete_group,
            set_selected,
            start,
            stop
        ])
        .run(tauri::generate_context!())
        .expect("error while running Luma");
}
