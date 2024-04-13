pub mod library;
pub mod reader;

use crate::prelude::*;

#[tauri::command]
pub async fn close_current_window(webview: WebviewWindow) -> Result<()> {
  debug!(name = "close_current_window");
  webview.close().map_err(Into::into)
}

#[tauri::command]
pub async fn focus_main_window(app: AppHandle) -> Result<()> {
  debug!(name = "focus_main_window");
  app
    .get_main_window()?
    .set_focus()
    .map_err(Into::into)
}
