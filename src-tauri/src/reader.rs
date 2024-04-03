use crate::book::ActiveBook;
use crate::prelude::*;
use crate::utils::event::Event;
use crate::utils::webview;
use tauri::{WebviewWindowBuilder, WindowEvent};

pub type BookMap = Arc<Mutex<HashMap<u16, (ActiveBook, WebviewWindow)>>>;

pub struct Reader {
  app: AppHandle,
  books: BookMap,
  id: u16,
}

impl Reader {
  pub fn new(app: &AppHandle) -> Self {
    Self {
      app: app.clone(),
      books: Arc::new(Mutex::new(HashMap::new())),
      id: 0,
    }
  }

  pub fn books(&self) -> BookMap {
    Arc::clone(&self.books)
  }

  pub async fn open_book(&mut self, book: ActiveBook) -> Result<()> {
    let books = self.books.lock().await;
    if let Some((_, webview)) = books.values().find(|(b, _)| b == &book) {
      webview.set_focus()?;
      return Ok(());
    }

    drop(books);

    self.id += 1;
    let id = self.id;

    let url = webview::reader_url();
    let dir = webview::reader_dir(&self.app, id)?;
    let label = webview::reader_label(id);

    let webview = WebviewWindowBuilder::new(&self.app, label, url)
      .data_directory(dir)
      .title(&book.title)
      .maximized(true)
      .resizable(true)
      .visible(false)
      .build()?;

    self.set_webview_listeners(&webview, id);
    self.set_webview_events(&webview, id);

    let mut books = self.books.lock().await;
    books.insert(id, (book, webview));

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
    let Some(focused) = self.get_focused_book().await else {
      return Ok(());
    };

    // We must make sure the key still exists after acquiring the lock.
    let books = self.books.lock().await;
    if books.len() < 2 || !books.contains_key(&focused) {
      return Ok(());
    }

    let id = books
      .keys()
      .cycle()
      .skip_while(|id| **id != focused)
      .skip(1)
      .find(|id| books.contains_key(id));

    let Some(id) = id else {
      return Ok(());
    };

    if let Some((_, webview)) = books.get(id) {
      return webview.set_focus().map_err(Into::into);
    }

    Ok(())
  }

  async fn get_focused_book(&self) -> Option<u16> {
    let books = self.books.lock().await;
    for (id, (_, webview)) in books.iter() {
      if webview.is_focused().unwrap_or(false) {
        return Some(*id);
      }
    }

    None
  }

  pub async fn get_book_as_value(&self, id: u16) -> Option<Value> {
    let books = self.books.lock().await;
    let book = books.get(&id).map(|(b, _)| b)?;
    Some(book.as_value())
  }

  fn set_webview_listeners(&self, webview: &WebviewWindow, id: u16) {
    let handle = self.app.clone();
    webview.listen(Event::WillMountReader.to_string(), move |_| {
      let label = webview::reader_label(id);
      if let Some(webview) = handle.get_webview_window(&label) {
        let js = format!("window.__KOTORI__ = {{ readerId: {id} }}");
        webview.eval(&js).ok();
      }
    });
  }

  fn set_webview_events(&self, webview: &WebviewWindow, id: u16) {
    let books = Arc::clone(&self.books);
    webview.on_window_event(move |event| {
      if matches!(event, WindowEvent::Destroyed) {
        // They are captured by the closure, but we need to clone before moving into the async block.
        // Otherwise, it wouldn't be possible to call the closure more than once.
        let books = Arc::clone(&books);
        async_runtime::spawn(async move {
          let mut books = books.lock().await;
          books.remove(&id);
        });
      }
    });
  }
}
