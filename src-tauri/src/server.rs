use crate::book::ActiveBook;
use crate::prelude::*;
use crate::{windows, VERSION};
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
      let router = Router::new()
        .route("/library/:book/cover", get(book_cover))
        .route("/reader", get(reader_root))
        .route("/reader/:book/:page", get(book_page))
        .with_state(app);

      let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
      axum::serve(listener, router).await.unwrap();
    });
  });
}

async fn reader_root(State(app): State<AppHandle>) -> Html<String> {
  let windows = windows!(&app);
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

async fn book_cover(State(app): State<AppHandle>, Path(book): Path<i32>) -> Response {
  if let Ok(book) = ActiveBook::from_id(&app, book).await {
    return match book.get_cover_as_bytes().await {
      Ok(bytes) => (StatusCode::OK, bytes).into_response(),
      Err(err) => err.into_response(),
    };
  };

  err!(BookNotFound).into_response()
}

async fn book_page(
  State(app): State<AppHandle>,
  Path((book, page)): Path<(u16, usize)>,
) -> Response {
  let windows = windows!(&app);
  let windows = windows.read().await;
  if let Some(window) = windows.get(&book) {
    return match window.book.get_page_as_bytes(page).await {
      Ok(bytes) => (StatusCode::OK, bytes).into_response(),
      Err(err) => err.into_response(),
    };
  };

  err!(BookNotFound).into_response()
}
