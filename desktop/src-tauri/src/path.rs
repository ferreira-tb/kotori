use crate::err;
use crate::result::Result;
use std::path::{Path, PathBuf};
use tauri::path::PathResolver;
use tauri::Wry;

pub trait PathExt {
  /// Open path with the default application using a detached process.
  fn open_detached(&self) -> Result<()>;
  /// Open the parent directory of the path with the default application using a detached process.
  fn open_parent_detached(&self) -> Result<()>;

  fn try_parent(&self) -> Result<&Path>;
  fn try_str(&self) -> Result<&str>;
  fn try_string(&self) -> Result<String>;
}

impl<P: AsRef<Path>> PathExt for P {
  fn open_detached(&self) -> Result<()> {
    let path = self.as_ref();
    open::that_detached(path).map_err(Into::into)
  }

  fn open_parent_detached(&self) -> Result<()> {
    self.try_parent()?.open_detached()
  }

  fn try_parent(&self) -> Result<&Path> {
    let path = self.as_ref();
    path
      .parent()
      .ok_or_else(|| err!(InvalidPath, "{}", path.display()))
  }

  fn try_str(&self) -> Result<&str> {
    let path = self.as_ref();
    path
      .to_str()
      .ok_or_else(|| err!(InvalidPath, "{}", path.display()))
  }

  fn try_string(&self) -> Result<String> {
    self.try_str().map(ToOwned::to_owned)
  }
}

pub trait PathResolverExt {
  fn cover(&self, book_id: i32) -> PathBuf;
  fn cover_dir(&self) -> PathBuf;

  #[cfg(feature = "devtools")]
  fn dev_cache_dir(&self) -> PathBuf;
  #[cfg(feature = "devtools")]
  fn mocks_dir(&self) -> PathBuf;
}

impl PathResolverExt for PathResolver<Wry> {
  /// Path to the cover image of a book.
  fn cover(&self, book_id: i32) -> PathBuf {
    self.cover_dir().join(book_id.to_string())
  }

  fn cover_dir(&self) -> PathBuf {
    self
      .app_cache_dir()
      .expect("failed to resolve app cache dir")
      .join("covers")
  }

  #[cfg(feature = "devtools")]
  fn dev_cache_dir(&self) -> PathBuf {
    self
      .app_cache_dir()
      .expect("failed to resolve app cache dir")
      .join("dev-cache")
  }

  #[cfg(feature = "devtools")]
  fn mocks_dir(&self) -> PathBuf {
    self.dev_cache_dir().join("mocks")
  }
}
