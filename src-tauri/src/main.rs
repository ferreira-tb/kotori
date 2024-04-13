// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod book;
mod command;
mod database;
mod error;
mod event;
mod library;
mod macros;
mod menu;
mod prelude;
mod reader;
mod server;
mod utils;

use error::BoxResult;
use reader::Reader;
use sea_orm::DatabaseConnection;
use tauri::async_runtime::RwLock;
use tauri::{App, AppHandle, Manager, WindowEvent};
use utils::app::AppHandleExt;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Kotori {
  pub db: DatabaseConnection,
  pub reader: RwLock<Reader>,
}

fn main() {
  tauri::Builder::default()
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_http::init())
    .plugin(tauri_plugin_manatsu::init())
    .plugin(tauri_plugin_persisted_scope::init())
    .plugin(tauri_plugin_shell::init())
    .plugin(tauri_plugin_window_state::Builder::default().build())
    .setup(setup)
    .invoke_handler(tauri::generate_handler![
      command::close_current_window,
      command::focus_main_window,
      command::library::add_to_library_from_dialog,
      command::library::get_library_books,
      command::library::show_library_book_context_menu,
      command::library::update_book_rating,
      command::reader::delete_book_page,
      command::reader::get_current_reader_book,
      command::reader::get_current_reader_window_id,
      command::reader::open_book,
      command::reader::open_book_from_dialog,
      command::reader::show_reader_page_context_menu,
      command::reader::switch_reader_focus,
    ])
    .run(tauri::generate_context!())
    .expect("could not start kotori");
}

fn setup(app: &mut App) -> BoxResult<()> {
  let handle = app.handle();
  let kotori = Kotori {
    db: database::connect(handle)?,
    reader: RwLock::new(Reader::new(handle)),
  };

  app.manage(kotori);

  let menu = menu::main::build(handle)?;
  let main_window = handle.get_main_window()?;
  main_window.set_menu(menu)?;

  main_window.on_menu_event(menu::main::on_event(handle));
  main_window.on_window_event(on_main_window_event(handle));

  // This depends on state managed by Tauri, so it MUST be called after `app.manage`.
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
