use crate::image::create_thumbnail;
use crate::prelude::*;
use image::ImageFormat;

#[derive(Clone, Debug)]
pub enum Cover {
  Extracted(PathBuf),
  NotExtracted,
}

impl Cover {
  pub fn from_id(app: &AppHandle, id: i32) -> Result<Self> {
    let path = app.path().cover(id)?;
    if path.try_exists()? {
      return Ok(path.into());
    }

    Ok(Cover::NotExtracted)
  }

  pub async fn extract(path: &Path, buf: Vec<u8>, format: ImageFormat) -> Result<Self> {
    let path = path.to_path_buf();
    let task = spawn_blocking(move || {
      create_thumbnail(buf, format, &path)?;
      Ok(path.into())
    });

    task.await?
  }

  pub fn path(&self) -> Option<&Path> {
    match self {
      Self::Extracted(path) => Some(path),
      Self::NotExtracted => None,
    }
  }

  pub fn path_buf(&self) -> Option<PathBuf> {
    self.path().map(ToOwned::to_owned)
  }
}

impl<P: AsRef<Path>> From<P> for Cover {
  fn from(path: P) -> Self {
    let path = path.as_ref().to_path_buf();
    Self::Extracted(path)
  }
}
