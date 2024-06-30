use std::sync::OnceLock;
use std::thread;

use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::Router;
use serde::Deserialize;
use tokio::net::TcpListener;
use tokio::sync::oneshot;

use crate::book::ActiveBook;
use crate::prelude::*;
use crate::window::WindowManager;

static PORT: OnceLock<u16> = OnceLock::new();

/// This depends on state managed by Tauri.
pub fn serve(app: &AppHandle) -> Result<()> {
  let app = app.clone();
  let (tx, rx) = oneshot::channel();

  thread::spawn(move || {
    block_on(async move {
      let router = Router::new()
        .route("/kotori/library/:book_id", get(book_cover))
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
