pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("book already exists")]
  AlreadyExists,
  #[error("book cache dir not found")]
  CacheNotFound,
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
  Unknown(#[from] anyhow::Error),
}

impl serde::Serialize for Error {
  fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
  where
    S: serde::ser::Serializer,
  {
    serializer.serialize_str(self.to_string().as_ref())
  }
}
