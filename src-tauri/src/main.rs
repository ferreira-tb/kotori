// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::module_name_repetitions)]

mod command;
pub mod database;
pub mod error;
mod event;
mod library;
mod menu;
pub mod prelude;
mod state;
mod utils;

use library::Library;
use state::{Database, Kotori, BOOK_CACHE};
use std::fs;
use tauri::Manager;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
  tauri::Builder::default()
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_persisted_scope::init())
    .plugin(tauri_plugin_window_state::Builder::default().build())
    .setup(|app| {
      let book_cache = app.path().app_cache_dir()?.join("books");
      if let Ok(false) = book_cache.try_exists() {
        fs::create_dir_all(&book_cache)?;
      }

      BOOK_CACHE.set(book_cache).unwrap();

      app.manage(Database {
        conn: database::connect(app).unwrap(),
      });

      app.manage(Kotori {
        library: Mutex::new(Library::new()),
      });

      let menu = menu::build(app).unwrap();
      app.set_menu(menu)?;
      app.on_menu_event(menu::event_handler);

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      command::add_to_library_with_dialog,
      command::open_with_dialog,
      command::version
    ])
    .run(tauri::generate_context!())
    .expect("could not start kotori");
}
