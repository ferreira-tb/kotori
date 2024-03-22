// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod book;
mod command;
pub mod database;
pub mod error;
pub mod prelude;
mod utils;

use book::Book;
use sea_orm::DatabaseConnection;
use tauri::menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder};
use tauri::Manager;
use tokio::sync::Mutex;
use tokio::task;

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

      app.set_menu(menu)?;

      app.on_menu_event(|handle, event| match event.id.0.as_str() {
        "open_book" => {
          let handle = handle.clone();
          task::spawn(async move {
            Book::open(&handle).await.unwrap();
          });
        }
        _ => {}
      });

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      command::open_book,
      command::version
    ])
    .run(tauri::generate_context!())
    .expect("could not start kotori");
}
