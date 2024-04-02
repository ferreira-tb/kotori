use crate::book::ActiveBook;
use crate::event::Event;
use crate::prelude::*;
use crate::utils::webview;
use axum::routing::get;
use axum::Router;
use indoc::formatdoc;
use tauri::{WebviewWindow, WebviewWindowBuilder, WindowEvent};
use tokio::net::TcpListener;

pub struct Reader {
  app: AppHandle,
  books: Arc<RwLock<HashMap<u16, (ActiveBook, WebviewWindow)>>>,
  current_id: u16,
}

impl Reader {
  pub fn new(app: AppHandle) -> Self {
    Self {
      app,
      books: Arc::new(RwLock::new(HashMap::new())),
      current_id: 0,
    }
  }

  pub fn serve(&self) {
    thread::spawn(|| {
      async_runtime::spawn(async {
        let router = Router::new().route("/", get(root));

        let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
        axum::serve(listener, router).await.unwrap();
      })
    });
  }

  pub async fn open_book(&mut self, book: ActiveBook) -> Result<()> {
    let books = self.books.read().await;
    if let Some((_, webview)) = books.values().find(|(b, _)| b == &book) {
      webview.set_focus()?;
      return Ok(());
    }

    drop(books);

    self.current_id += 1;
    let id = self.current_id;

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

    let handle = self.app.clone();
    webview.listen(Event::WillMountReader.to_string(), move |_| {
      let label = webview::reader_label(id);
      if let Some(webview) = handle.get_webview_window(&label) {
        let js = formatdoc! {"
          window.__KOTORI__ = {{
            readerId: {id}
          }};
        "};

        webview.eval(&js).ok();
      }
    });

    let books = Arc::clone(&self.books);
    webview.on_window_event(move |event| {
      if matches!(event, WindowEvent::Destroyed) {
        // They are captured by the closure, but we need to clone before moving into the async block.
        // Otherwise, it wouldn't be possible to call the closure more than once.
        let books = Arc::clone(&books);
        async_runtime::spawn(async move {
          let mut books = books.write().await;
          books.remove(&id);
        });
      }
    });

    let mut books = self.books.write().await;
    books.insert(id, (book, webview));

    Ok(())
  }
}

async fn root() -> &'static str {
  "Hello, World!"
}
