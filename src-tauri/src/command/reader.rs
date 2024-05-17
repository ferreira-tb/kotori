use crate::book::{self, ActiveBook, ReaderBook};
use crate::{prelude::*, reader};

#[tauri::command]
pub async fn delete_page_with_dialog(app: AppHandle, window_id: u16, page: usize) -> Result<()> {
  debug!(command = "delete_page", window_id, page);
  reader::delete_page_with_dialog(&app, window_id, page).await
}

#[tauri::command]
pub async fn get_current_reader_book(app: AppHandle, window_id: u16) -> Result<ReaderBook> {
  debug!(command = "get_current_reader_book", window_id);
  ReaderBook::from_reader(&app, window_id).await
}

#[tauri::command]
pub async fn show_reader_page_context_menu(
  window: Window,
  window_id: u16,
  page: usize,
) -> Result<()> {
  use crate::menu::context::reader::page::{self, Context, ReaderPageContextMenu};

  debug!(
    command = "show_reader_page_context_menu",
    window = window.label(),
    window_id,
    page
  );

  let app = window.app_handle();
  let windows = app.reader_windows();
  let windows = windows.read().await;
  let Some(reader_window) = windows.get(&window_id) else {
    return Ok(());
  };

  let book_id = reader_window.book.id_or_try_init(app).await.ok();
  let ctx = Context { window_id, book_id, page };

  if let Some(state) = window.try_state::<ReaderPageContextMenu>() {
    *state.ctx.lock().unwrap() = ctx;
    window.popup_menu(&state.menu)?;
  } else {
    let menu = page::build(app)?;
    window.popup_menu(&menu)?;

    let ctx = std::sync::Mutex::new(ctx);
    window.manage(ReaderPageContextMenu { menu, ctx });
  }

  Ok(())
}

#[tauri::command]
pub async fn switch_reader_focus(app: AppHandle) -> Result<()> {
  debug!(command = "switch_reader_focus");
  reader::switch_focus(&app).await
}

#[tauri::command]
pub async fn open_book(app: AppHandle, book_id: i32) -> Result<()> {
  debug!(command = "open_book", book_id);
  let book = ActiveBook::from_id(&app, book_id).await?;
  book.open(&app).await
}

#[tauri::command]
pub async fn open_book_from_dialog(app: AppHandle) -> Result<()> {
  debug!(command = "open_book_from_dialog");
  book::open_from_dialog(&app).await
}

#[tauri::command]
pub async fn toggle_reader_menu(webview: WebviewWindow) -> Result<()> {
  debug!(command = "toggle_reader_menu", window = webview.label());
  if webview.is_menu_visible()? {
    webview.hide_menu().map_err(Into::into)
  } else {
    webview.show_menu().map_err(Into::into)
  }
}
