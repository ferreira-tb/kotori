// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod book;
mod command;
pub mod database;
pub mod error;
mod events;
mod library;
pub mod prelude;
mod state;
mod utils;

use events::menu_event_handler;
use state::Kotori;
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
      app.manage(Kotori {
        books: Mutex::new(Vec::default()),
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
