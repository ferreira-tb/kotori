use crate::book::ActiveBook;
use crate::event::Event;
use crate::prelude::*;
use crate::utils::collections::OrderedMap;
use crate::window::ReaderWindow;
use tauri_plugin_dialog::{DialogExt, MessageDialogBuilder, MessageDialogKind};

pub type WindowMap = Arc<RwLock<OrderedMap<u16, ReaderWindow>>>;
pub struct Reader {
  windows: WindowMap,
}

impl Reader {
  pub fn new() -> Self {
    let windows = OrderedMap::<u16, ReaderWindow>::default();
    let windows = Arc::new(RwLock::new(windows));
    Self { windows }
  }

  pub fn windows(&self) -> WindowMap {
    Arc::clone(&self.windows)
  }
}

impl Default for Reader {
  fn default() -> Self {
    Self::new()
  }
}

pub async fn open_book(app: &AppHandle, book: ActiveBook) -> Result<()> {
  {
    let windows = app.reader_windows();
    let windows = windows.read().await;
    if let Some(window) = windows.values().find(|w| w.book == book) {
      return window.webview.set_focus().map_err(Into::into);
    }
  }

  let (id, window) = ReaderWindow::open(app, book)?;
  let windows = app.reader_windows();
  let mut windows = windows.write().await;
  windows.insert(id, window);

  Ok(())
}

pub async fn open_many<I>(app: &AppHandle, books: I) -> Result<()>
where
  I: IntoIterator<Item = ActiveBook>,
{
  for book in books {
    open_book(app, book).await?;
  }

  Ok(())
}

pub async fn close_all(app: &AppHandle) -> Result<()> {
  let windows = app.reader_windows();
  for window in windows.read().await.values() {
    let _ = window.webview.close();
  }

  Ok(())
}

pub async fn close_others(app: &AppHandle, window_id: u16) -> Result<()> {
  let windows = app.reader_windows();
  for window in windows.read().await.values() {
    if window.id != window_id {
      let _ = window.webview.close();
    }
  }

  Ok(())
}

pub async fn get_window_id_by_label(app: &AppHandle, label: &str) -> Option<u16> {
  let windows = app.reader_windows();
  let windows = windows.read().await;
  windows
    .iter()
    .find(|(_, window)| window.webview.label() == label)
    .map(|(_, window)| window.id)
}

async fn get_focused_window_id(app: &AppHandle) -> Option<u16> {
  let windows = app.reader_windows();
  let windows = windows.read().await;
  for window in windows.values() {
    if window.webview.is_focused().unwrap_or(false) {
      return Some(window.id);
    }
  }

  None
}

pub async fn switch_focus(app: &AppHandle) -> Result<()> {
  let main_window = app.main_window();
  if main_window.is_focused().unwrap_or(false) {
    let windows = app.reader_windows();
    let windows = windows.read().await;
    if let Some((_, window)) = windows.first() {
      return window.webview.set_focus().map_err(Into::into);
    }
  }

  let Some(focused) = get_focused_window_id(app).await else {
    return Ok(());
  };

  let windows = app.reader_windows();
  let windows = windows.read().await;
  if windows.len() < 2 || !windows.contains_key(&focused) {
    return Ok(());
  }

  let window = windows
    .values()
    .cycle()
    .skip_while(|window| window.id != focused)
    .skip(1)
    .find(|window| windows.contains_key(&window.id));

  if let Some(window) = window {
    if let Some(window) = windows.get(&window.id) {
      return window.webview.set_focus().map_err(Into::into);
    }
  };

  Ok(())
}

pub async fn delete_page(app: &AppHandle, window_id: u16, page: usize) -> Result<()> {
  let windows = app.reader_windows();
  let mut windows = windows.write().await;

  if let Some(window) = windows.get_mut(&window_id) {
    let book = window.book.clone();
    book.delete_page(app, page).await?;

    let pages = window.book.reload_pages().await?;
    if pages.is_empty() {
      info!("book \"{}\" is empty, closing window", window.book.title);
      return window.webview.close().map_err(Into::into);
    }

    Event::PageDeleted { window_id }.emit(app)?;
  }

  Ok(())
}

pub async fn delete_page_with_dialog(app: &AppHandle, window_id: u16, page: usize) -> Result<()> {
  let (tx, rx) = oneshot::channel();
  let dialog = app.dialog().clone();

  let message = "Are you sure you want to delete this page?";
  MessageDialogBuilder::new(dialog, "Delete page", message)
    .kind(MessageDialogKind::Warning)
    .ok_button_label("Delete")
    .cancel_button_label("Cancel")
    .show(move |response| {
      let _ = tx.send(response);
    });

  if let Ok(true) = rx.await {
    delete_page(app, window_id, page).await?;
  }

  Ok(())
}
