use crate::image::create_thumbnail;
use crate::prelude::*;
use image::ImageFormat;

#[derive(Clone, Debug)]
pub enum Cover {
  Extracted(PathBuf),
  NotExtracted,
}

impl Cover {
  pub async fn extract<P>(path: P, buf: Vec<u8>, format: ImageFormat) -> Result<Self>
  where
    P: AsRef<Path>,
  {
    create_thumbnail(buf, format, &path).await?;
    Ok(path.into())
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
