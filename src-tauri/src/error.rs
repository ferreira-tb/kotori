use serde::ser::Serializer;
use serde::Serialize;

pub type Result<T> = std::result::Result<T, Error>;
pub type BoxResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("book is empty")]
  Empty,
  #[error("{0}")]
  InvalidBook(String),
  #[error("{0}")]
  InvalidPage(String),

  #[error(transparent)]
  Database(#[from] sea_orm::error::DbErr),
  #[error(transparent)]
  Glob(#[from] globset::Error),
  #[error(transparent)]
  Io(#[from] std::io::Error),
  #[error(transparent)]
  Json(#[from] serde_json::Error),
  #[error(transparent)]
  Tauri(#[from] tauri::Error),
  #[error(transparent)]
  TokioJoin(#[from] tokio::task::JoinError),
  #[error(transparent)]
  TokioRecv(#[from] tokio::sync::oneshot::error::RecvError),
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
    serializer.serialize_str(self.to_string().as_ref())
  }
}

#[macro_export]
macro_rules! err {
  ($e:ident) => {
    Error::$e
  };
  ($e:ident, $($arg:tt)*) => {
    Error::$e(format!($($arg)*))
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
