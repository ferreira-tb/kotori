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
    .plugin(tauri_plugin_persisted_scope::init())
    .plugin(tauri_plugin_window_state::Builder::default().build())
    .setup(|app| {
      let kotori = Kotori {
        books: Mutex::new(Vec::default()),
        database: database::connect(app).unwrap(),
      };

      app.manage(kotori);

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      command::open_file,
      command::version
    ])
    .run(tauri::generate_context!())
    .expect("could not start kotori");
}
