use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::ser::Serializer;
use serde::Serialize;

pub type Result<T> = std::result::Result<T, Error>;
pub type BoxResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("book not found")]
  BookNotFound,
  #[error("book is empty")]
  EmptyBook,
  #[error("{0}")]
  InvalidBook(String),
  #[error("invalid path: {0}")]
  InvalidPath(String),
  #[error("rating must be an integer between 0 and 5")]
  InvalidRating,
  #[error("page not found")]
  PageNotFound,
  #[error("window not found: {0}")]
  WindowNotFound(String),

  #[error(transparent)]
  ChronoParse(#[from] chrono::ParseError),
  #[error(transparent)]
  Database(#[from] sea_orm::error::DbErr),
  #[error(transparent)]
  Glob(#[from] globset::Error),
  #[error(transparent)]
  Image(#[from] image::ImageError),
  #[error(transparent)]
  Io(#[from] std::io::Error),
  #[error(transparent)]
  Json(#[from] serde_json::Error),
  #[error(transparent)]
  Manatsu(#[from] tauri_plugin_manatsu::Error),
  #[error(transparent)]
  Tauri(#[from] tauri::Error),
  #[error(transparent)]
  TauriStore(#[from] tauri_plugin_store::Error),
  #[error(transparent)]
  TauriWindowState(#[from] tauri_plugin_window_state::Error),
  #[error(transparent)]
  TokioAcquire(#[from] tokio::sync::AcquireError),
  #[error(transparent)]
  TokioJoin(#[from] tokio::task::JoinError),
  #[error(transparent)]
  TokioRecv(#[from] tokio::sync::oneshot::error::RecvError),
  #[error(transparent)]
  TryFromInt(#[from] std::num::TryFromIntError),
  #[error(transparent)]
  WalkDir(#[from] walkdir::Error),
  #[error(transparent)]
  Unknown(#[from] anyhow::Error),
  #[error(transparent)]
  Zip(#[from] zip::result::ZipError),
}

impl Serialize for Error {
  fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(self.to_string().as_str())
  }
}

impl IntoResponse for Error {
  fn into_response(self) -> Response {
    let status = match self {
      Error::BookNotFound | Error::PageNotFound | Error::WindowNotFound(_) => StatusCode::NOT_FOUND,
      _ => StatusCode::INTERNAL_SERVER_ERROR,
    };

    (status, self.to_string()).into_response()
  }
}

#[macro_export]
macro_rules! err {
  ($e:ident) => {
    $crate::error::Error::$e
  };
  ($e:ident, $($arg:tt)*) => {
    $crate::error::Error::$e(format!($($arg)*))
  };
}

#[macro_export]
macro_rules! bail {
  ($e:ident) => {
    return Err($crate::err!($e));
  };
  ($e:ident, $($arg:tt)*) => {
    return Err($crate::err!($e, $($arg)*));
  };
}
