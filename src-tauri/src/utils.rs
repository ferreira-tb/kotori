use crate::prelude::*;
use serde::ser::Serializer;
use serde::Serialize;
use tempfile::tempdir_in;

pub mod date {
  use chrono::Local;

  /// <https://docs.rs/chrono/latest/chrono/format/strftime/index.html>
  pub const TIMESTAMP: &str = "%F %T%.3f %:z";

  pub fn now() -> String {
    Local::now().format(TIMESTAMP).to_string()
  }
}

pub mod glob {
  use globset::{Glob, GlobBuilder, GlobSet, GlobSetBuilder};

  fn glob(glob: &str) -> Glob {
    GlobBuilder::new(glob)
      .case_insensitive(true)
      .build()
      .unwrap()
  }

  pub fn book() -> GlobSet {
    GlobSetBuilder::new()
      .add(glob("*.cbr"))
      .add(glob("*.cbz"))
      .build()
      .unwrap()
  }

  pub fn book_page() -> GlobSet {
    GlobSetBuilder::new()
      .add(glob("*.gif"))
      .add(glob("*.jpg"))
      .add(glob("*.jpeg"))
      .add(glob("*.png"))
      .add(glob("*.webp"))
      .build()
      .unwrap()
  }
}

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
