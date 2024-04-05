use crate::prelude::*;
use crate::reader::WindowMap;
use crate::VERSION;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use indoc::formatdoc;
use tokio::net::TcpListener;

pub fn serve(app: &AppHandle) {
  let app = app.clone();
  thread::spawn(move || {
    async_runtime::block_on(async move {
      let reader_windows = {
        let kotori = app.state::<Kotori>();
        let reader = kotori.reader.read().await;
        reader.windows()
      };

      let router = Router::new()
        .route("/library/:book/cover", get(book_cover))
        .route("/reader", get(reader_root))
        .route("/reader/:book/:page", get(book_page))
        .with_state(reader_windows);

      let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
      axum::serve(listener, router).await.unwrap();
    });
  });
}

async fn reader_root(State(windows): State<WindowMap>) -> Html<String> {
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

async fn book_cover(State(windows): State<WindowMap>, Path(book): Path<u16>) -> Response {
  let windows = windows.read().await;
  if let Some(window) = windows.get(&book) {
    return match window.book.get_cover_as_bytes().await {
      Ok(bytes) => (StatusCode::OK, bytes).into_response(),
      Err(err) => err.into_response(),
    };
  };

  err!(BookNotFound).into_response()
}

async fn book_page(
  State(windows): State<WindowMap>,
  Path((book, page)): Path<(u16, usize)>,
) -> Response {
  let windows = windows.read().await;
  if let Some(window) = windows.get(&book) {
    return match window.book.get_page_as_bytes(page).await {
      Ok(bytes) => (StatusCode::OK, bytes).into_response(),
      Err(err) => err.into_response(),
    };
  };

  err!(BookNotFound).into_response()
}
