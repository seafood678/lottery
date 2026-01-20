#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;
mod models;
mod storage;

use commands::AppStateWrapper;
use models::AppState;
use std::sync::Mutex;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .manage(AppStateWrapper {
            state: Mutex::new(AppState::default()),
        })
        .setup(|app| {
            let state = app.state::<AppStateWrapper>();
            if let Ok(loaded) = storage::load_state(app.config().as_ref()) {
                let mut app_state = state.state.lock().unwrap();
                *app_state = loaded;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::add_person,
            commands::add_persons_batch,
            commands::remove_person,
            commands::get_all_persons,
            commands::get_available_persons,
            commands::add_prize,
            commands::update_prize,
            commands::remove_prize,
            commands::get_all_prizes,
            commands::draw_lottery,
            commands::revoke_winner,
            commands::get_all_winners,
            commands::export_winners_json,
            commands::export_winners_text,
            commands::save_data,
            commands::load_data,
            commands::reset_all_data,
            commands::clear_winners,
            commands::get_app_title,
            commands::set_app_title,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
