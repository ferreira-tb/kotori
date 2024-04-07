use crate::book::{ActiveBook, IntoValue, ReaderBook};
use crate::prelude::*;
use crate::utils::{window, OrderedMap};
use futures::future::try_join_all;
use std::sync::atomic::{self, AtomicU16};
use tauri::{WebviewWindowBuilder, WindowEvent};

/// Counter for window IDs.
static ID: AtomicU16 = AtomicU16::new(0);

pub type WindowMap = Arc<RwLock<OrderedMap<u16, ReaderWindow>>>;

pub struct Reader {
  app: AppHandle,
  windows: WindowMap,
}

impl Reader {
  pub fn new(app: &AppHandle) -> Self {
    let windows = OrderedMap::<u16, ReaderWindow>::default();
    Self {
      app: app.clone(),
      windows: Arc::new(RwLock::new(windows)),
    }
  }

  pub fn windows(&self) -> WindowMap {
    Arc::clone(&self.windows)
  }

  pub async fn open_book(&self, book: ActiveBook) -> Result<()> {
    let windows = self.windows.read().await;
    if let Some(window) = windows.values().find(|w| w.book == book) {
      window.webview.set_focus()?;
      return Ok(());
    }

    drop(windows);

    let window_id = ID.fetch_add(1, atomic::Ordering::SeqCst);

    let url = window::webview_url("reader");
    let dir = window::dir(&self.app, format!("reader/{window_id}"))?;
    let label = format!("reader-{window_id}");

    let webview = WebviewWindowBuilder::new(&self.app, label, url)
      .data_directory(dir)
      .title(book.title.to_string())
      .maximized(true)
      .resizable(true)
      .visible(false)
      .build()?;

    self.set_window_events(&webview, window_id);

    let mut windows = self.windows.write().await;
    let window = ReaderWindow { book, webview };
    windows.insert(window_id, window);

    Ok(())
  }

  pub async fn open_many<I>(&self, books: I) -> Result<()>
  where
    I: IntoIterator<Item = ActiveBook>,
  {
    let books = books.into_iter().map(|book| self.open_book(book));
    try_join_all(books).await?;

    Ok(())
  }

  pub async fn switch_focus(&self) -> Result<()> {
    let main_window = self.app.get_webview_window("main");
    if matches!(main_window, Some(it) if it.is_focused().unwrap_or(false)) {
      let windows = self.windows.read().await;
      if let Some((_, window)) = windows.first() {
        return window.webview.set_focus().map_err(Into::into);
      }
    }

    let Some(focused) = self.get_focused_window_id().await else {
      return Ok(());
    };

    let windows = self.windows.read().await;
    if windows.len() < 2 || !windows.contains_key(&focused) {
      return Ok(());
    }

    let id = windows
      .keys()
      .cycle()
      .skip_while(|id| **id != focused)
      .skip(1)
      .find(|id| windows.contains_key(*id));

    let Some(id) = id else {
      return Ok(());
    };

    if let Some(window) = windows.get(id) {
      return window.webview.set_focus().map_err(Into::into);
    }

    Ok(())
  }

  async fn get_focused_window_id(&self) -> Option<u16> {
    let windows = self.windows.read().await;
    for (id, window) in &*windows {
      if window.webview.is_focused().unwrap_or(false) {
        return Some(*id);
      }
    }

    None
  }

  pub async fn get_book_as_value(&self, window_id: u16) -> Option<Value> {
    self
      .windows
      .read()
      .await
      .get(&window_id)
      .map(ReaderBook::from_reader_window)?
      .into_value()
      .await
      .ok()
  }

  pub async fn get_window_id_by_label(&self, label: &str) -> Option<u16> {
    let windows = self.windows.read().await;
    windows
      .iter()
      .find(|(_, window)| window.webview.label() == label)
      .map(|(id, _)| *id)
  }

  fn set_window_events(&self, webview: &WebviewWindow, window_id: u16) {
    let windows = self.windows();
    webview.on_window_event(move |event| {
      if matches!(event, WindowEvent::CloseRequested { .. }) {
        let windows = Arc::clone(&windows);
        async_runtime::spawn(async move {
          let mut windows = windows.write().await;
          windows.shift_remove(&window_id);
        });
      }
    });
  }
}

pub struct ReaderWindow {
  pub book: ActiveBook,
  webview: WebviewWindow,
}
