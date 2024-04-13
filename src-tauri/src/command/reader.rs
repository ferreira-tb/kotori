use crate::book::{self, ActiveBook};
use crate::event::Event;
use crate::{prelude::*, reader};

#[tauri::command]
pub async fn delete_book_page(app: AppHandle, webview: WebviewWindow, page: usize) -> Result<()> {
  let window_id = reader::get_window_id(&app, webview.label()).await?;

  let kotori = app.kotori();
  let reader = kotori.reader.read().await;
  reader
    .delete_book_page(window_id, page)
    .await
    .inspect_err(|err| error!("\"{err}\""))
}

#[tauri::command]
pub async fn get_current_reader_book(app: AppHandle, webview: WebviewWindow) -> Result<Json> {
  let window_id = reader::get_window_id(&app, webview.label()).await?;

  let kotori = app.kotori();
  let reader = kotori.reader.read().await;
  reader
    .get_book_as_json(window_id)
    .await
    .ok_or_else(|| err!(BookNotFound))
    .inspect_err(|err| error!("\"{err}\""))
}

#[tauri::command]
pub async fn get_current_reader_window_id(app: AppHandle, webview: WebviewWindow) -> Result<u16> {
  reader::get_window_id(&app, webview.label())
    .await
    .inspect_err(|err| error!("\"{err}\""))
}

#[tauri::command]
pub async fn request_delete_page(app: AppHandle, window_id: u16, page: usize) -> Result<()> {
  Event::DeletePageRequested { window_id, page }.emit(&app)
}

#[tauri::command]
pub async fn show_reader_page_context_menu(
  app: AppHandle,
  window: Window,
  window_id: u16,
  page: usize,
) -> Result<()> {
  use crate::menu::context::reader::page;

  let windows = reader::get_windows(&app).await;
  let windows = windows.read().await;

  if let Some(reader_window) = windows.get(&window_id) {
    let book_id = reader_window.book.id_or_try_init(&app).await;
    let menu = page::build(&app, book_id)?;
    window.on_menu_event(page::on_event(&app, window_id, book_id, page));

    menu.popup(window)?;
  }

  Ok(())
}

#[tauri::command]
pub async fn switch_reader_focus(app: AppHandle) -> Result<()> {
  let kotori = app.kotori();
  let reader = kotori.reader.read().await;
  reader
    .switch_focus()
    .await
    .inspect_err(|err| error!("\"{err}\""))
}

#[tauri::command]
pub async fn open_book(app: AppHandle, id: i32) -> Result<()> {
  let book = ActiveBook::from_id(&app, id).await?;
  book
    .open(&app)
    .await
    .inspect_err(|err| error!("\"{err}\""))
}

#[tauri::command]
pub async fn open_book_from_dialog(app: AppHandle) -> Result<()> {
  book::open_from_dialog(&app)
    .await
    .inspect_err(|err| error!("\"{err}\""))
}
