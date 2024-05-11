pub mod library;
pub mod reader;

use crate::prelude::*;

#[tauri::command]
pub async fn close_window(window: WebviewWindow) -> Result<()> {
  debug!(command = "close_window", label = window.label());
  window.close().map_err(Into::into)
}

#[tauri::command]
pub async fn focus_main_window(app: AppHandle) -> Result<()> {
  debug!(command = "focus_main_window");
  app
    .get_main_window()
    .set_focus()
    .map_err(Into::into)
}

#[tauri::command]
pub async fn show_window(window: WebviewWindow) -> Result<()> {
  debug!(command = "show_window", label = window.label());
  window.show().map_err(Into::into)
}

#[tauri::command]
pub async fn toggle_fullscreen(window: WebviewWindow) -> Result<()> {
  debug!(command = "toggle_fullscreen", label = window.label());
  let is_fullscreen = window.is_fullscreen()?;
  window
    .set_fullscreen(!is_fullscreen)
    .map_err(Into::into)
}
