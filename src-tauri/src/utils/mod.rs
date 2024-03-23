pub mod glob;
pub mod date;

use crate::prelude::*;
use serde::ser::Serializer;
use serde::Serialize;
use tempfile::tempdir_in;

#[derive(Debug)]
pub struct TempDir(tempfile::TempDir);

impl TempDir {
  pub fn new() -> Result<Self> {
    let book_cache = BOOK_CACHE.get().unwrap();
    let temp_dir = tempdir_in(book_cache)?;
    Ok(Self(temp_dir))
  }

  pub fn path(&self) -> &Path {
    self.0.path()
  }
}

impl Serialize for TempDir {
  fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let path = self.0.path();
    let path = path.to_str().ok_or_else(|| {
      let err = format!("invalid path: {path:?}");
      serde::ser::Error::custom(err)
    })?;

    serializer.serialize_str(path)
  }
}
