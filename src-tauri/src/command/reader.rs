use crate::book::ActiveBook;
use crate::prelude::*;

#[tauri::command]
pub async fn get_current_reader_book(app: AppHandle, window: WebviewWindow) -> Result<Json> {
  let kotori = app.state::<Kotori>();
  let reader = kotori.reader.read().await;

  let label = window.label();
  let id = reader
    .get_window_id_by_label(label)
    .await
    .ok_or_else(|| err!(WindowNotFound, "{label}"))?;

  reader
    .get_book_as_value(id)
    .await
    .ok_or_else(|| err!(BookNotFound))
}

#[tauri::command]
pub async fn get_current_reader_window_id(app: AppHandle, window: WebviewWindow) -> Result<u16> {
  let kotori = app.state::<Kotori>();
  let reader = kotori.reader.read().await;

  let label = window.label();
  reader
    .get_window_id_by_label(label)
    .await
    .ok_or_else(|| err!(WindowNotFound, "{label}"))
}

#[tauri::command]
pub async fn open_book(app: AppHandle, id: i32) -> Result<()> {
  let book = ActiveBook::from_id(&app, id).await?;
  book.open(&app).await
}

#[tauri::command]
pub async fn open_book_from_dialog(app: AppHandle) -> Result<()> {
  ActiveBook::open_from_dialog(&app).await
}

#[tauri::command]
pub async fn switch_reader_focus(app: AppHandle) -> Result<()> {
  let kotori = app.state::<Kotori>();
  let reader = kotori.reader.read().await;
  reader.switch_focus().await
}
