use crate::book::{self, ActiveBook};
use crate::{prelude::*, reader};

#[tauri::command]
pub async fn delete_page_with_dialog(
  app: AppHandle,
  webview: WebviewWindow,
  page: usize,
) -> Result<()> {
  let label = webview.label();
  debug!(command = "delete_page", window = label, page);

  reader::get_window_id(&app, label)
    .and_then(|window_id| reader::delete_page_with_dialog(&app, window_id, page))
    .await
}

#[tauri::command]
pub async fn get_current_reader_book(app: AppHandle, webview: WebviewWindow) -> Result<Json> {
  let label = webview.label();
  debug!(command = "get_current_reader_book", window = label);
  let window_id = reader::get_window_id(&app, label).await?;
  
  reader::get_book_as_json(&app, window_id)
    .await
    .ok_or_else(|| err!(BookNotFound))
}

#[tauri::command]
pub async fn show_reader_page_context_menu(
  app: AppHandle,
  window: Window,
  window_id: u16,
  page: usize,
) -> Result<()> {
  use crate::menu::context::reader::page;
  use tauri::menu::ContextMenu;

  debug!(command = "show_reader_page_context_menu", window_id, page);
  let windows = reader::get_windows(&app);
  let windows = windows.read().await;

  if let Some(reader_window) = windows.get(&window_id) {
    let book_id = reader_window.book.id_or_try_init(&app).await.ok();
    let menu = page::build(&app, book_id)?;
    window.on_menu_event(page::on_event(&app, window_id, book_id, page));

    menu.popup(window)?;
  }

  Ok(())
}

#[tauri::command]
pub async fn switch_reader_focus(app: AppHandle) -> Result<()> {
  debug!(command = "switch_reader_focus");
  reader::switch_focus(&app).await
}

#[tauri::command]
pub async fn open_book(app: AppHandle, id: i32) -> Result<()> {
  debug!(command = "open_book", book_id = id);
  let book = ActiveBook::from_id(&app, id).await?;
  book.open(&app).await
}

#[tauri::command]
pub async fn open_book_from_dialog(app: AppHandle) -> Result<()> {
  debug!(command = "open_book_from_dialog");
  book::open_from_dialog(&app).await
}
