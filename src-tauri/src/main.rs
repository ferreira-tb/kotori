// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(let_chains, try_blocks)]

mod book;
mod command;
mod database;
mod error;
mod event;
mod image;
mod library;
mod menu;
mod prelude;
mod reader;
mod server;
mod utils;
mod window;

use book::BookHandle;
use error::BoxResult;
use reader::Reader;
use sea_orm::DatabaseConnection;
use tauri::{App, Manager};

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Kotori {
  pub db: DatabaseConnection,
  pub book_handle: BookHandle,
  pub reader: Reader,
}

fn main() {
  tauri::Builder::default()
    .plugin(tauri_plugin_clipboard_manager::init())
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_http::init())
    .plugin(tauri_plugin_manatsu::init())
    .plugin(tauri_plugin_persisted_scope::init())
    .plugin(tauri_plugin_shell::init())
    .plugin(tauri_plugin_store::Builder::new().build())
    .plugin(plugin::prevent_default())
    .plugin(plugin::single_instance())
    .plugin(plugin::window_state())
    .setup(setup)
    .invoke_handler(tauri::generate_handler![
      command::close_window,
      command::focus_main_window,
      command::notify_config_update,
      command::server_port,
      command::show_window,
      command::toggle_fullscreen,
      command::collection::get_collections,
      command::library::add_to_library_with_dialog,
      command::library::get_library_books,
      command::library::remove_book,
      command::library::remove_book_with_dialog,
      command::library::show_library_book_context_menu,
      command::library::update_book_rating,
      command::reader::delete_page_with_dialog,
      command::reader::get_current_reader_book,
      command::reader::open_book,
      command::reader::open_book_with_dialog,
      command::reader::show_reader_page_context_menu,
      command::reader::switch_reader_focus,
      command::reader::toggle_reader_menu
    ])
    .run(tauri::generate_context!())
    .expect("could not start kotori");
}

fn setup(app: &mut App) -> BoxResult<()> {
  let app = app.handle();

  #[cfg(any(debug_assertions, feature = "devtools"))]
  utils::log::setup_tracing(app);

  app.manage(Kotori {
    db: database::connect(app)?,
    book_handle: BookHandle::new(),
    reader: Reader::new(),
  });

  server::serve(app)?;
  window::app::create(app)?;

  Ok(())
}

mod plugin {
  use tauri::plugin::TauriPlugin;
  use tauri::Wry;

  use crate::window::{WindowExt, WindowManager};

  #[cfg(any(debug_assertions, feature = "devtools"))]
  pub fn prevent_default() -> TauriPlugin<Wry> {
    use tauri_plugin_prevent_default::Flags;

    tauri_plugin_prevent_default::Builder::new()
      .with_flags(Flags::all().difference(Flags::RELOAD))
      .build()
  }

  #[cfg(not(any(debug_assertions, feature = "devtools")))]
  pub fn prevent_default() -> TauriPlugin<Wry> {
    tauri_plugin_prevent_default::Builder::new().build()
  }

  pub fn single_instance() -> TauriPlugin<Wry> {
    tauri_plugin_single_instance::init(|app, _, _| {
      let _ = app.main_window().set_foreground_focus();
    })
  }

  pub fn window_state() -> TauriPlugin<Wry> {
    use tauri_plugin_window_state::StateFlags as Flags;

    let builder = tauri_plugin_window_state::Builder::new()
      .with_state_flags(Flags::MAXIMIZED | Flags::POSITION | Flags::SIZE)
      .map_label(|label| {
        if label.starts_with("reader") {
          "reader"
        } else {
          label
        }
      });

    builder.build()
  }
}
