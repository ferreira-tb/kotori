// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod book;
mod command;
pub mod database;
pub mod error;
pub mod prelude;

use sea_orm::DatabaseConnection;
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::Manager;

pub struct Kotori {
  pub books: Mutex<Vec<book::Book>>,
  pub db: DatabaseConnection,
}

pub type State<'a> = tauri::State<'a, Kotori>;

#[tokio::main]
async fn main() {
  tauri::Builder::default()
    .plugin(tauri_plugin_persisted_scope::init())
    .plugin(tauri_plugin_window_state::Builder::default().build())
    .plugin(tauri_plugin_manatsu::init())
    .setup(|app| {
      let config = Arc::clone(&app.config());
      let handle = thread::spawn(move || {
        tauri::async_runtime::block_on(database::connect(config)).unwrap()
      });

      let kotori = Kotori {
        books: Mutex::new(Vec::default()),
        db: handle.join().unwrap(),
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
