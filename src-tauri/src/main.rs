// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::module_name_repetitions)]

mod command;
pub mod database;
mod error;
mod event;
mod library;
mod menu;
mod prelude;
mod reader;
mod state;
mod utils;

use library::Library;
use prelude::*;
use reader::Reader;
use std::fs;
use tauri::WindowEvent;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
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

      let reader = Reader::new(app.handle().clone());
      reader.serve();

      let kotori = Kotori {
        db: database::connect(app).unwrap(),
        library: Mutex::new(Library::new()),
        reader,
      };

      app.manage(kotori);

      let menu = menu::build(app).unwrap();
      let main_window = app.get_webview_window("main").unwrap();
      main_window.set_menu(menu)?;

      let handle = app.handle().clone();
      main_window.on_menu_event(menu::event_handler(handle));

      let handle = app.handle().clone();
      main_window.on_window_event(move |event| {
        if matches!(event, WindowEvent::Destroyed) {
          handle.cleanup_before_exit();
          handle.exit(0);
        }
      });

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
