use std::{sync::OnceLock, thread};

use axum::{
  extract::{Json, Path, State},
  http::StatusCode,
  response::{Html, IntoResponse, Response},
  routing::{get, post},
  Router,
};
use indoc::formatdoc;
use serde::Deserialize;
use tokio::{net::TcpListener, sync::oneshot};

use crate::{book::ActiveBook, prelude::*, window::WindowManager, VERSION};

static PORT: OnceLock<u16> = OnceLock::new();

/// This depends on state managed by Tauri.
pub fn serve(app: &AppHandle) -> Result<()> {
  let app = app.clone();
  let (tx, rx) = oneshot::channel();

  thread::spawn(move || {
    block_on(async move {
      let router = Router::new()
        .route("/kotori/library/:book_id/cover", get(book_cover))
        .route("/kotori/reader", get(reader_root))
        .route("/kotori/reader/:window_id", post(book_page))
        .with_state(app);

      let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
      let port = listener
        .local_addr()
        .inspect(|it| info!(local_addr = %it))
        .unwrap()
        .port();

      let _ = tx.send(port);

      axum::serve(listener, router).await.unwrap();
    });
  });

  let port = block_on(rx)?;
  let _ = PORT.set(port);

  Ok(())
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

#[derive(Deserialize)]
struct BookPage {
  name: String,
}

async fn book_page(
  State(app): State<AppHandle>,
  Path(window_id): Path<u16>,
  Json(page): Json<BookPage>,
) -> Response {
  let windows = app.reader_windows();
  let windows = windows.read().await;
  if let Some(window) = windows.get(&window_id) {
    return match window.book.get_page_as_bytes(&page.name).await {
      Ok(bytes) => (StatusCode::OK, bytes).into_response(),
      Err(err) => err.into_response(),
    };
  };

  err!(BookNotFound).into_response()
}
