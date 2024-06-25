use crate::book::ActiveBook;
use crate::{prelude::*, VERSION};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use indoc::formatdoc;
use std::thread;
use tokio::net::TcpListener;
use tokio::sync::{oneshot, OnceCell};

static PORT: OnceCell<u16> = OnceCell::const_new();

/// This depends on state managed by Tauri.
pub fn serve(app: &AppHandle) {
  let app = app.clone();
  let (tx, rx) = oneshot::channel();

  thread::spawn(move || {
    async_runtime::block_on(async move {
      let router = Router::new()
        .route("/kotori/library/:book_id/cover", get(book_cover))
        .route("/kotori/reader", get(reader_root))
        .route("/kotori/reader/:window_id/:page", get(book_page))
        .with_state(app);

      let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
      let local_addr = listener
        .local_addr()
        .inspect(|it| info!(local_addr = %it))
        .unwrap();

      tx.send(local_addr.port()).unwrap();

      axum::serve(listener, router).await.unwrap();
    });
  });

  async_runtime::block_on(rx)
    .map(|port| PORT.set(port).unwrap())
    .unwrap();
}

pub fn port() -> u16 {
  *PORT.get().unwrap()
}

async fn reader_root(State(app): State<AppHandle>) -> Html<String> {
  let windows = app.reader_windows();
  let windows = windows.read().await;
  let amount = windows.len();

  macro_rules! row {
    ($($td:expr),*) => {{
      let mut row = String::from("<tr>");
      $(row.push_str(&format!("<td>{}</td>", $td));)*
      row.push_str("</tr>");
      row
    }};
  }

  let table = windows
    .iter()
    .sorted_unstable_by_key(|(id, _)| *id)
    .map(|(id, window)| row!(id, window.book.title))
    .collect::<String>();

  let html = formatdoc! {"
    <html lang='en'>
      <head>
        <title>Kotori {VERSION}</title>
      </head>
      <body>
        <p>Active books: {amount}</p>
        <table>{table}</table>
      </body>
    </html>
  "};

  Html(html)
}

async fn book_cover(State(app): State<AppHandle>, Path(book_id): Path<i32>) -> Response {
  if let Ok(book) = ActiveBook::from_id(&app, book_id).await {
    return match book.get_cover_as_bytes(&app).await {
      Ok(bytes) => (StatusCode::OK, bytes).into_response(),
      Err(err) => err.into_response(),
    };
  };

  err!(BookNotFound).into_response()
}

async fn book_page(
  State(app): State<AppHandle>,
  Path((window_id, page)): Path<(u16, usize)>,
) -> Response {
  let windows = app.reader_windows();
  let windows = windows.read().await;
  if let Some(window) = windows.get(&window_id) {
    return match window.book.get_page_as_bytes(page).await {
      Ok(bytes) => (StatusCode::OK, bytes).into_response(),
      Err(err) => err.into_response(),
    };
  };

  err!(BookNotFound).into_response()
}
