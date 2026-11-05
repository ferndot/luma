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
fn rename_light(app: State<'_, Shared>, id: String, name: String) -> Result<(), String> {
    let mut s = lock(&app)?;
    logic::rename_light(&mut s, &id, &name)?;
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
                names: saved.names,
                api_key: saved.api_key,
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
            rename_light,
            set_api_key,
            set_selected,
            start,
            stop
        ])
        .run(tauri::generate_context!())
        .expect("error while running Luma");
}
