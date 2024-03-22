use crate::library::Library;
use crate::prelude::*;

#[tauri::command]
pub async fn version(app: AppHandle) -> String {
  app.config().version.clone().unwrap()
}

#[tauri::command]
pub async fn open_book(app: AppHandle) -> Result<()> {
  Library::open_book(&app).await
}

#[tauri::command]
pub async fn add_to_library(app: AppHandle) -> Result<()> {
  Library::add(&app).await
}
