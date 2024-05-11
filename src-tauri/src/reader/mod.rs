mod window;

pub use window::{get_window_id, label, ReaderWindow};

use crate::book::{ActiveBook, IntoJson, ReaderBook};
use crate::event::Event;
use crate::utils::collections::OrderedMap;
use crate::{prelude::*, utils};
use std::sync::atomic::{self, AtomicU16};
use tauri::{WebviewWindowBuilder, WindowEvent};
use tauri_plugin_dialog::{MessageDialogBuilder, MessageDialogKind};

static NEXT_WINDOW_ID: AtomicU16 = AtomicU16::new(0);

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

pub fn get_windows(app: &AppHandle) -> WindowMap {
  app.kotori().reader.windows()
}

pub async fn open_book(app: &AppHandle, book: ActiveBook) -> Result<()> {
  {
    let windows = get_windows(app);
    let windows = windows.read().await;
    if let Some(window) = windows.values().find(|w| w.book == book) {
      return window.webview.set_focus().map_err(Into::into);
    }
  }

  let window_id = NEXT_WINDOW_ID.fetch_add(1, atomic::Ordering::SeqCst);

  let label = window::label(window_id);
  let url = utils::window::webview_url("reader");
  let dir = utils::window::data_directory(app, &label)?;

  let script = format!("window.KOTORI = {{ readerWindowId: {window_id} }}");
  trace!(%script);

  let webview = WebviewWindowBuilder::new(app, label, url)
    .initialization_script(&script)
    .data_directory(dir)
    .title(book.title.to_string())
    .min_inner_size(800.0, 600.0)
    .resizable(true)
    .maximizable(true)
    .minimizable(true)
    .maximized(true)
    .visible(false)
    .build()?;

  set_window_event(app, &webview, window_id);

  let windows = get_windows(app);
  let mut windows = windows.write().await;
  windows.insert(window_id, ReaderWindow { book, webview });

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

pub async fn get_book_as_json(app: &AppHandle, window_id: u16) -> Option<Json> {
  let windows = get_windows(app);
  let windows = windows.read().await;
  windows
    .get(&window_id)
    .map(ReaderBook::from_reader_window)?
    .into_json()
    .await
    .ok()
}

async fn get_window_id_by_label(app: &AppHandle, label: &str) -> Option<u16> {
  let windows = get_windows(app);
  let windows = windows.read().await;
  windows
    .iter()
    .find(|(_, window)| window.webview.label() == label)
    .map(|(id, _)| *id)
}

fn set_window_event(app: &AppHandle, webview: &WebviewWindow, window_id: u16) {
  let app = app.clone();
  webview.on_window_event(move |event| {
    if matches!(event, WindowEvent::CloseRequested { .. }) {
      let app = app.clone();
      async_runtime::spawn(async move {
        let windows = get_windows(&app);
        let mut windows = windows.write().await;
        windows.shift_remove(&window_id);
      });
    }
  });
}

async fn get_focused_window_id(app: &AppHandle) -> Option<u16> {
  let windows = get_windows(app);
  let windows = windows.read().await;
  for (id, window) in windows.iter() {
    if window.webview.is_focused().unwrap_or(false) {
      return Some(*id);
    }
  }

  None
}

pub async fn switch_focus(app: &AppHandle) -> Result<()> {
  let main_window = app.get_main_window();
  if main_window.is_focused().unwrap_or(false) {
    let windows = get_windows(app);
    let windows = windows.read().await;
    if let Some((_, window)) = windows.first() {
      return window.webview.set_focus().map_err(Into::into);
    }
  }

  let Some(focused) = get_focused_window_id(app).await else {
    return Ok(());
  };

  let windows = get_windows(app);
  let windows = windows.read().await;
  if windows.len() < 2 || !windows.contains_key(&focused) {
    return Ok(());
  }

  let id = windows
    .keys()
    .cycle()
    .skip_while(|id| **id != focused)
    .skip(1)
    .find(|id| windows.contains_key(*id));

  if let Some(id) = id {
    if let Some(window) = windows.get(id) {
      return window.webview.set_focus().map_err(Into::into);
    }
  };

  Ok(())
}

pub async fn delete_page(app: &AppHandle, window_id: u16, page: usize) -> Result<()> {
  let windows = get_windows(app);
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
