// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![cfg(not(any(target_os = "android", target_os = "ios")))]
#![feature(let_chains, try_blocks)]

mod book;
mod command;
mod database;
mod error;
mod event;
mod fs;
mod image;
mod library;
mod macros;
mod manager;
mod menu;
mod path;
mod prelude;
mod reader;
mod result;
mod server;
mod utils;
mod window;

use manager::Kotori;
use result::{BoxResult, Result, ResultExt};
use tauri::App;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
  tauri::Builder::default()
    .plugin(tauri_plugin_clipboard_manager::init())
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_http::init())
    .plugin(tauri_plugin_manatsu::init())
    .plugin(tauri_plugin_persisted_scope::init())
    .plugin(tauri_plugin_pinia::init())
    .plugin(tauri_plugin_shell::init())
    .plugin(plugin::prevent_default())
    .plugin(plugin::single_instance())
    .plugin(plugin::window_state())
    .setup(setup)
    .invoke_handler(tauri::generate_handler![
      command::close_window,
      command::focus_main_window,
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
    ])
    .run(tauri::generate_context!())
    .expect("could not start kotori");
}

fn setup(app: &mut App) -> BoxResult<()> {
  let app = app.handle();
  let result: Result<()> = try {
    #[cfg(feature = "tracing")]
    utils::log::setup_tracing(app)?;

    Kotori::init(app)?;
    server::serve(app)?;
    window::app::open(app)?;
  };

  if result.is_err() {
    result.into_blocking_err_dialog(app);
    app.exit(1);
  }

  Ok(())
}

mod plugin {
  use crate::manager::ManagerExt;
  use crate::result::ResultExt;
  use crate::window::WindowExt;
  use tauri::plugin::TauriPlugin;
  use tauri::Wry;

  #[cfg(feature = "devtools")]
  pub fn prevent_default() -> TauriPlugin<Wry> {
    use tauri_plugin_prevent_default::Flags;

    tauri_plugin_prevent_default::Builder::new()
      .with_flags(Flags::all().difference(Flags::RELOAD))
      .build()
  }

  #[cfg(not(feature = "devtools"))]
  pub fn prevent_default() -> TauriPlugin<Wry> {
    tauri_plugin_prevent_default::Builder::new().build()
  }

  pub fn single_instance() -> TauriPlugin<Wry> {
    tauri_plugin_single_instance::init(|app, _, _| {
      app
        .main_window()
        .set_foreground_focus()
        .into_err_dialog(app);
    })
  }

  pub fn window_state() -> TauriPlugin<Wry> {
    use tauri_plugin_window_state::StateFlags as Flags;

    tauri_plugin_window_state::Builder::new()
      .with_state_flags(Flags::MAXIMIZED | Flags::POSITION | Flags::SIZE)
      .map_label(|label| {
        if label.starts_with("reader") {
          "reader"
        } else {
          label
        }
      })
      .build()
  }
}
