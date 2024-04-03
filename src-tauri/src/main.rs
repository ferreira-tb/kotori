// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::module_name_repetitions)]

mod book;
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

use error::BoxResult;
use library::Library;
use prelude::*;
use reader::Reader;
use tauri::{App, WindowEvent};
use utils::webview;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
  tauri::Builder::default()
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_persisted_scope::init())
    .plugin(tauri_plugin_window_state::Builder::default().build())
    .setup(setup)
    .invoke_handler(tauri::generate_handler![
      command::close_webview_window,
      command::get_active_book,
      command::switch_reader_focus
    ])
    .run(tauri::generate_context!())
    .expect("could not start kotori");
}

fn setup(app: &mut App) -> BoxResult<()> {
  let reader = Reader::new(app.handle().clone());
  reader.serve();

  let kotori = Kotori {
    db: database::connect(app).unwrap(),
    library: Mutex::new(Library::new()),
    reader: Mutex::new(reader),
  };

  app.manage(kotori);

  let menu = menu::build(app).unwrap();
  let main_window = app.get_webview_window("main").unwrap();
  main_window.set_menu(menu)?;

  let handle = app.handle().clone();
  main_window.on_menu_event(menu::on_menu_event(handle));

  let handle = app.handle().clone();
  main_window.on_window_event(on_main_window_event(handle));

  Ok(())
}

fn on_main_window_event(app: AppHandle) -> impl Fn(&WindowEvent) {
  move |event| {
    if matches!(event, WindowEvent::Destroyed) {
      // Cleanup the reader webview artifacts before exiting.
      let reader_dir = webview::dir(&app, "reader").unwrap();
      if let Ok(true) = reader_dir.try_exists() {
        std::fs::remove_dir_all(reader_dir).ok();
      }

      app.cleanup_before_exit();
      app.exit(0);
    }
  }
}
