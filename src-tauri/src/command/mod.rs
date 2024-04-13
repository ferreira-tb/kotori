pub mod library;
pub mod reader;

use crate::prelude::*;

#[tauri::command]
pub async fn close_current_window(webview: WebviewWindow) -> Result<()> {
  webview.close().map_err(Into::into)
}

#[tauri::command]
pub async fn focus_main_window(app: AppHandle) -> Result<()> {
  app
    .get_main_window()?
    .set_focus()
    .map_err(Into::into)
}
