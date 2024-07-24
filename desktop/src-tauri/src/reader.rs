use crate::book::ActiveBook;
use crate::event::Event;
use crate::prelude::*;
use crate::utils::collections::OrderedMap;
use crate::window::{ReaderWindow, WindowExt, WindowManager};
use std::sync::Arc;
use tauri_plugin_dialog::{DialogExt, MessageDialogBuilder, MessageDialogKind};
use tokio::sync::{oneshot, RwLock};

pub type WindowMap = Arc<RwLock<OrderedMap<u16, ReaderWindow>>>;

#[derive(Default)]
pub struct Reader {
  windows: WindowMap,
}

impl Reader {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn windows(&self) -> WindowMap {
    Arc::clone(&self.windows)
  }
}

pub async fn open_book(app: &AppHandle, book: ActiveBook) -> Result<()> {
  // If the book is already open, bring it to the foreground.
  {
    let windows = app.reader_windows();
    let windows = windows.read().await;
    let webview = windows
      .values()
      .find(|it| it.book == book)
      .and_then(|it| it.webview_window(app));

    if let Some(webview) = webview {
      return webview.set_foreground_focus();
    }
  }

  // Otherwise, open a new window.
  {
    let window = ReaderWindow::open(app, book)?;
    let windows = app.reader_windows();
    let mut windows = windows.write().await;
    windows.insert(window.id, window);
  }

  // This will read lock the windows.
  ReaderWindow::update_all_menus(app).await
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
    if let Some(webview) = window.webview_window(app) {
      webview.close().into_err_log(app);
    }
  }

  app.main_window().set_foreground_focus()
}

pub async fn close_others(app: &AppHandle, window_id: u16) -> Result<()> {
  let windows = app.reader_windows();
  for window in windows.read().await.values() {
    if window.id != window_id {
      if let Some(webview) = window.webview_window(app) {
        webview.close().into_err_log(app);
      }
    }
  }

  Ok(())
}

pub async fn get_book_path(app: &AppHandle, window_id: u16) -> Option<PathBuf> {
  let windows = app.reader_windows();
  let windows = windows.read().await;
  windows
    .get(&window_id)
    .map(|window| window.book.path.clone())
}

pub async fn get_window_id_by_label(app: &AppHandle, label: &str) -> Option<u16> {
  let windows = app.reader_windows();
  let windows = windows.read().await;
  windows
    .iter()
    .find(|(_, window)| {
      window
        .webview_window(app)
        .is_some_and(|it| it.label() == label)
    })
    .map(|(_, window)| window.id)
}

pub async fn switch_focus(app: &AppHandle) -> Result<()> {
  let main_window = app.main_window();
  if main_window.is_focused()? {
    let windows = app.reader_windows();
    let windows = windows.read().await;

    let webview = windows
      .first()
      .and_then(|(_, window)| window.webview_window(app));

    if let Some(webview) = webview {
      webview.set_foreground_focus()?;

      if main_window.is_fullscreen()? {
        webview.set_fullscreen(true)?;
      }

      return Ok(());
    }
  }

  drop(main_window);

  if let Some(focused) = app.get_focused_window()
    && let Some(focused_id) = get_window_id_by_label(app, focused.label()).await
  {
    let windows = app.reader_windows();
    let windows = windows.read().await;

    // Having the id of the focused window should be enough indication that it exists on the map.
    // That said, it isn't possible to know at what point in time we will acquire the read lock.
    // Therefore, it's safer to check the existence of the id before continuing.
    if windows.len() >= 2 && windows.contains_key(&focused_id) {
      let webview = windows
        .values()
        .cycle()
        .skip_while(|window| window.id != focused_id)
        .skip(1)
        .find(|window| windows.contains_key(&window.id))
        .and_then(|window| windows.get(&window.id))
        .and_then(|window| window.webview_window(app));

      if let Some(webview) = webview {
        webview.set_foreground_focus()?;

        if focused.is_fullscreen()? {
          webview.set_fullscreen(true)?;
        }
      };
    }
  }

  Ok(())
}

pub async fn delete_page(app: &AppHandle, window_id: u16, name: &str) -> Result<()> {
  let windows = app.reader_windows();
  let mut windows = windows.write().await;

  if let Some(window) = windows.get_mut(&window_id) {
    window.book.delete_page(name).await?;

    if window.book.pages().await?.is_empty() {
      if let Some(webview) = window.webview_window(app) {
        return webview.close().map_err(Into::into);
      }
    }

    Event::PageDeleted { window_id, name }.emit(app)?;
  }

  Ok(())
}

pub async fn delete_page_with_dialog(app: &AppHandle, window_id: u16, name: &str) -> Result<()> {
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
    delete_page(app, window_id, name).await?;
  }

  Ok(())
}
