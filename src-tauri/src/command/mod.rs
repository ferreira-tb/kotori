pub mod reader;

use crate::prelude::*;

#[tauri::command]
pub async fn close_current_window(webview: WebviewWindow) -> Result<()> {
  webview.close().map_err(Into::into)
}

#[tauri::command]
pub async fn focus_main_window(app: AppHandle) -> Result<()> {
  if let Some(window) = app.get_webview_window("main") {
    return window.set_focus().map_err(Into::into);
  };

  Ok(())
}
