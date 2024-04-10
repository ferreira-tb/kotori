use crate::book::ActiveBook;
use crate::prelude::*;
use crate::reader::Reader;

#[tauri::command]
pub async fn get_current_reader_book(app: AppHandle, webview: WebviewWindow) -> Result<Json> {
  let window_id = Reader::get_window_id(&app, &webview).await?;

  let kotori = app.state::<Kotori>();
  let reader = kotori.reader.read().await;
  reader
    .get_book_as_json(window_id)
    .await
    .ok_or_else(|| err!(BookNotFound))
}

#[tauri::command]
pub async fn get_current_reader_window_id(app: AppHandle, webview: WebviewWindow) -> Result<u16> {
  Reader::get_window_id(&app, &webview).await
}

#[tauri::command]
pub async fn show_reader_page_context_menu(
  app: AppHandle,
  window: Window,
  webview: WebviewWindow,
  page: usize,
) -> Result<()> {
  use crate::menu::context::reader::page;

  let window_id = Reader::get_window_id(&app, &webview).await?;
  let windows = Reader::get_windows(&app).await;
  let windows = windows.read().await;

  if let Some(reader_window) = windows.get(&window_id) {
    let book_id = reader_window.book.id_or_try_init(&app).await;
    let menu = page::build(&app, book_id)?;

    if let Some(id) = book_id {
      window.on_menu_event(page::on_event(&app, id, page));
    }

    menu.popup(window)?;
  }

  Ok(())
}

#[tauri::command]
pub async fn switch_reader_focus(app: AppHandle) -> Result<()> {
  let kotori = app.state::<Kotori>();
  let reader = kotori.reader.read().await;
  reader.switch_focus().await
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
