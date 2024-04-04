use crate::prelude::*;

#[tauri::command]
pub async fn close_webview_window(webview: WebviewWindow) -> Result<()> {
  webview.close().map_err(Into::into)
}

#[tauri::command]
pub async fn focus_main_window(app: AppHandle) -> Result<()> {
  if let Some(window) = app.get_webview_window("main") {
    return window.set_focus().map_err(Into::into);
  };

  Ok(())
}

#[tauri::command]
pub async fn get_active_book(app: AppHandle, id: u16) -> Result<Value> {
  let kotori = app.state::<Kotori>();
  let reader = kotori.reader.lock().await;

  reader
    .get_book_as_value(id)
    .await
    .ok_or_else(|| err!(BookNotFound))
}

#[tauri::command]
pub async fn switch_reader_focus(app: AppHandle) -> Result<()> {
  let kotori = app.state::<Kotori>();
  let reader = kotori.reader.lock().await;
  reader.switch_focus().await
}
