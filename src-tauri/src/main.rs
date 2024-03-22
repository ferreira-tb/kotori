// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod book;
mod command;
pub mod database;
pub mod error;
mod events;
pub mod prelude;
mod utils;

use book::Book;
use events::menu_event_handler;
use sea_orm::DatabaseConnection;
use tauri::menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder};
use tauri::Manager;
use tokio::sync::Mutex;

pub struct Kotori {
  pub books: Mutex<Vec<Book>>,
  pub database: DatabaseConnection,
}

pub type State<'a> = tauri::State<'a, Kotori>;

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
      command::open_book,
      command::version
    ])
    .run(tauri::generate_context!())
    .expect("could not start kotori");
}
