pub mod library;
pub mod reader;

use crate::prelude::*;

#[tauri::command]
pub async fn close_window(webview: WebviewWindow) -> Result<()> {
  debug!(name = "close_window");
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

#[tauri::command]
pub async fn toggle_fullscreen(webview: WebviewWindow) -> Result<()> {
  debug!(name = "toggle_fullscreen");
  let is_fullscreen = webview.is_fullscreen()?;
  webview
    .set_fullscreen(!is_fullscreen)
    .map_err(Into::into)
}
