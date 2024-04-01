use crate::library::Library;
use crate::prelude::*;

#[tauri::command]
pub async fn version() -> String {
  crate::VERSION.to_string()
}

#[tauri::command]
pub async fn open_with_dialog(app: AppHandle) -> Result<()> {
  Library::open_with_dialog(&app).await
}

#[tauri::command]
pub async fn add_to_library_with_dialog(app: AppHandle) -> Result<()> {
  Library::add_with_dialog(&app).await
}
