// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod command;
pub mod error;
pub mod prelude;

fn main() {
  tauri::Builder::default()
    .plugin(tauri_plugin_persisted_scope::init())
    .plugin(tauri_plugin_window_state::Builder::default().build())
    .plugin(tauri_plugin_manatsu::init())
    .invoke_handler(tauri::generate_handler![command::version])
    .run(tauri::generate_context!())
    .expect("could not start kotori");
}
