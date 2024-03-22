// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

mod command;
pub mod database;
pub mod error;
mod events;
mod library;
pub mod prelude;
mod state;
mod utils;

use events::menu_event_handler;
use library::Library;
use state::{Kotori, BOOK_CACHE};
use std::fs;
use tauri::menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder};
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

      app.manage(Kotori {
        library: Mutex::new(Library::new()),
        database: database::connect(app).unwrap(),
      });

      let menu = MenuBuilder::new(app).build()?;

      menu.append(
        &SubmenuBuilder::new(app, "File")
          .item(&MenuItemBuilder::with_id("open_book", "Open file").build(app)?)
          .item(&MenuItemBuilder::with_id("add_to_library", "Add to library").build(app)?)
          .separator()
          .quit()
          .build()?,
      )?;

      menu.append(
        &SubmenuBuilder::new(app, "Browse")
          .item(
            &MenuItemBuilder::with_id("library", "Library")
              .accelerator("F1")
              .build(app)?,
          )
          .build()?,
      )?;

      app.set_menu(menu)?;
      app.on_menu_event(menu_event_handler);

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      command::add_to_library,
      command::open_book,
      command::version
    ])
    .run(tauri::generate_context!())
    .expect("could not start kotori");
}
