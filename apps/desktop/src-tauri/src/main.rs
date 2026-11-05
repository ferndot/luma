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
        logic::save_remembered(&s);
    }
    Ok(logic::view(&s))
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
            // Lights found in earlier runs are kept in the app's data folder.
            let store_path = app.path().app_data_dir().ok().map(|dir| dir.join("lights.json"));
            let remembered = store_path.as_deref().map(logic::load_remembered).unwrap_or_default();
            app.manage(Mutex::new(AppState { store_path, remembered, ..Default::default() }));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_state, set_mode, find_lights, set_selected, start, stop])
        .run(tauri::generate_context!())
        .expect("error while running Luma");
}
