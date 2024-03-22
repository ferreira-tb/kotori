use crate::prelude::*;

#[tauri::command]
pub async fn version(app: AppHandle) -> String {
  app.config().version.clone().unwrap()
}

#[tauri::command]
pub async fn open_book(app: AppHandle) -> Result<()> {
  Book::open(&app).await
}
