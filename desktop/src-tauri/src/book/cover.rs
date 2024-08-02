use crate::prelude::*;

#[derive(Clone, Debug)]
pub enum Cover {
  Extracted(PathBuf),
  NotExtracted,
}

impl Cover {
  pub fn from_book_id(app: &AppHandle, id: i32) -> Self {
    let path = app.path().cover(id);
    if let Ok(true) = path.try_exists() {
      return Self::Extracted(path);
    }

    Self::NotExtracted
  }

  pub fn into_path(self) -> Option<PathBuf> {
    match self {
      Self::Extracted(path) => Some(path),
      Self::NotExtracted => None,
    }
  }
}

impl<P: AsRef<Path>> From<P> for Cover {
  fn from(path: P) -> Self {
    let path = path.as_ref().to_path_buf();
    Self::Extracted(path)
  }
}
