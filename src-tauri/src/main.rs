// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::module_name_repetitions)]

mod book;
mod command;
pub mod database;
mod error;
mod library;
mod menu;
mod prelude;
mod reader;
mod server;
mod utils;

use error::BoxResult;
use library::Library;
use reader::Reader;
use sea_orm::DatabaseConnection;
use tauri::async_runtime::Mutex;
use tauri::{App, AppHandle, Manager, WindowEvent};

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Kotori {
  pub db: DatabaseConnection,
  pub library: Mutex<Library>,
  pub reader: Mutex<Reader>,
}

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
  let handle = app.handle();
  let kotori = Kotori {
    db: database::connect(handle).unwrap(),
    library: Mutex::new(Library::new()),
    reader: Mutex::new(Reader::new(handle)),
  };

  app.manage(kotori);

  let menu = menu::build(handle).unwrap();
  let main_window = app.get_webview_window("main").unwrap();
  main_window.set_menu(menu)?;

  main_window.on_menu_event(menu::on_menu_event(handle));
  main_window.on_window_event(on_main_window_event(handle));

  // This depends on state managed by tauri, so it must be called after `app.manage`.
  server::serve(handle);

  Ok(())
}

fn on_main_window_event(app: &AppHandle) -> impl Fn(&WindowEvent) {
  let app = app.clone();
  move |event| {
    if matches!(event, WindowEvent::Destroyed) {
      app.cleanup_before_exit();
      app.exit(0);
    }
  }
}
