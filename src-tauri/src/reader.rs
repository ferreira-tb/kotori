use crate::book::ActiveBook;
use crate::prelude::*;
use crate::utils::event::Event;
use crate::utils::webview;
use tauri::{WebviewWindowBuilder, WindowEvent};

pub type WindowMap = Arc<RwLock<HashMap<u16, ReaderWindow>>>;

pub struct Reader {
  app: AppHandle,
  windows: WindowMap,
  current_id: u16,
}

impl Reader {
  pub fn new(app: &AppHandle) -> Self {
    Self {
      app: app.clone(),
      windows: Arc::new(RwLock::new(HashMap::new())),
      current_id: 0,
    }
  }

  pub fn windows(&self) -> WindowMap {
    Arc::clone(&self.windows)
  }

  pub async fn open_book(&mut self, book: ActiveBook) -> Result<()> {
    let windows = self.windows.read().await;
    if let Some(window) = windows.values().find(|w| w.book == book) {
      window.webview.set_focus()?;
      return Ok(());
    }

    drop(windows);

    self.current_id += 1;
    let window_id = self.current_id;

    let url = webview::reader_url();
    let dir = webview::reader_dir(&self.app, window_id)?;
    let label = webview::reader_label(window_id);

    let webview = WebviewWindowBuilder::new(&self.app, label, url)
      .data_directory(dir)
      .title(&book.title)
      .maximized(true)
      .resizable(true)
      .visible(false)
      .build()?;

    self.set_webview_listeners(&webview, window_id);
    self.set_webview_events(&webview, window_id);

    let mut windows = self.windows.write().await;
    let window = ReaderWindow { book, webview };
    windows.insert(window_id, window);

    Ok(())
  }

  pub async fn open_many<I>(&mut self, books: I) -> Result<()>
  where
    I: IntoIterator<Item = ActiveBook>,
  {
    for book in books {
      self.open_book(book).await?;
    }

    Ok(())
  }

  pub async fn switch_focus(&self) -> Result<()> {
    let Some(focused) = self.get_focused_window_id().await else {
      return Ok(());
    };

    // We must make sure the key still exists after acquiring the lock.
    let windows = self.windows.read().await;
    if windows.len() < 2 || !windows.contains_key(&focused) {
      return Ok(());
    }

    let id = windows
      .keys()
      .cycle()
      .skip_while(|id| **id != focused)
      .skip(1)
      .find(|id| windows.contains_key(id));

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
    for (id, window) in windows.iter() {
      if window.webview.is_focused().unwrap_or(false) {
        return Some(*id);
      }
    }

    None
  }

  pub async fn get_book_as_value(&self, window_id: u16) -> Option<Value> {
    let windows = self.windows.read().await;
    windows
      .get(&window_id)
      .map(|window| window.book.as_value())
  }

  fn set_webview_listeners(&self, webview: &WebviewWindow, window_id: u16) {
    let handle = self.app.clone();
    let label = webview::reader_label(window_id);
    webview.listen(Event::WillMountReader, move |_| {
      if let Some(webview) = handle.get_webview_window(&label) {
        let js = format!("window.__KOTORI__ = {{ readerId: {window_id} }}");
        webview.eval(&js).ok();
      }
    });
  }

  fn set_webview_events(&self, webview: &WebviewWindow, window_id: u16) {
    let windows = self.windows();
    webview.on_window_event(move |event| {
      if matches!(event, WindowEvent::Destroyed) {
        let windows = Arc::clone(&windows);
        async_runtime::spawn(async move {
          let mut windows = windows.write().await;
          windows.remove(&window_id);
        });
      }
    });
  }
}

pub struct ReaderWindow {
  pub book: ActiveBook,
  webview: WebviewWindow,
}
