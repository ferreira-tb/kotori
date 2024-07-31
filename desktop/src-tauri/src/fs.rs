use crate::prelude::*;
use std::fs::{self, File};
use uuid::Uuid;

/// Temporary file that is deleted when dropped.
pub struct Tempfile {
  pub path: PathBuf,
  pub file: File,
}

impl Tempfile {
  /// Create a new temporary file in the specified directory.
  pub fn new_in(dir: impl AsRef<Path>) -> Result<Self> {
    let path = dir.as_ref().join(Self::filename());
    let file = File::create(&path)?;
    Ok(Self { path, file })
  }

  fn filename() -> String {
    format!("{}.kotori", Uuid::now_v7())
  }
}

impl Drop for Tempfile {
  fn drop(&mut self) {
    if let Ok(true) = self.path.try_exists() {
      let _ = fs::remove_file(&self.path);
    }

    #[cfg(feature = "tracing")]
    tracing::trace!(tempfile_drop = %self.path.display());
  }
}
