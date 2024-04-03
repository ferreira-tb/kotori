use crate::book::ActiveBook;
use crate::event::Event;
use crate::prelude::*;
use crate::utils::webview;
use crate::VERSION;
use axum::extract::{Path, State};
use axum::http::{HeaderValue, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use indoc::formatdoc;
use tauri::{WebviewWindowBuilder, WindowEvent};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

type Books = Arc<Mutex<HashMap<u16, (ActiveBook, WebviewWindow)>>>;

pub struct Reader {
  app: AppHandle,
  books: Books,
  id: u16,
}

impl Reader {
  const URL: &'static str = "0.0.0.0:3000";

  pub fn new(app: AppHandle) -> Self {
    Self {
      app,
      books: Arc::new(Mutex::new(HashMap::new())),
      id: 0,
    }
  }

  pub fn serve(&self) {
    let books = Arc::clone(&self.books);
    thread::spawn(|| {
      async_runtime::block_on(async {
        let mut router = Router::new()
          .route("/", get(root))
          .route("/reader/:book/:page", get(book_page))
          .with_state(books);

        if tauri::dev() {
          let origin = HeaderValue::from_static("http://localhost:1422");
          let layer = CorsLayer::new().allow_origin(origin);
          router = router.layer(layer);
        }

        let listener = TcpListener::bind(Self::URL).await.unwrap();
        axum::serve(listener, router).await.unwrap();
      });
    });
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

  pub async fn switch_reader_focus(&self) -> Result<()> {
    let Some(focused) = self.get_focused_book().await else {
      return Ok(());
    };

    // Better we make sure the key still exists after acquiring the lock.
    // Would be nasty being trapped forever inside the while loops.
    let books = self.books.lock().await;
    if books.len() < 2 || !books.contains_key(&focused) {
      return Ok(());
    }

    // Move the iterator to the focused book.
    let mut keys = books.keys().cycle();
    while let Some(id) = keys.next() {
      if id == &focused {
        break;
      }
    }

    while let Some(id) = keys.next() {
      if let Some((_, webview)) = books.get(id) {
        return webview.set_focus().map_err(Into::into);
      }
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

    let mut pages = book.file.pages.keys().copied().collect_vec();
    pages.sort_unstable();

    let json = json!({
      "path": book.path,
      "title": book.title,
      "pages": pages
    });

    Some(json)
  }

  fn set_webview_listeners(&self, webview: &WebviewWindow, id: u16) {
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

async fn root(State(books): State<Books>) -> Html<String> {
  let books = books.lock().await;
  let amount = books.len();

  let html = formatdoc! {"
    <html lang='en'>
      <head>
        <meta charset='UTF-8'>
        <meta name='viewport' content='width=device-width, initial-scale=1.0'>
        <title>Kotori {VERSION}</title>
      </head>
      <body>
        <h1>Kotori Reader</h1>
        <p>Books: {amount}</p>
      </body>
    </html>
  "};

  Html(html)
}

async fn book_page(State(books): State<Books>, Path((book, page)): Path<(u16, usize)>) -> Response {
  let mut books = books.lock().await;
  let book = books.get_mut(&book).map(|(b, _)| b);

  let Some(book) = book else {
    return err!(BookNotFound).into_response();
  };

  match book.file.get_page_as_bytes(page) {
    Ok(bytes) => (StatusCode::OK, bytes).into_response(),
    Err(err) => err.into_response(),
  }
}
