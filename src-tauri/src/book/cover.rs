use crate::prelude::*;
use image::codecs::webp::WebPEncoder;
use image::io::Reader as ImageReader;
use image::ImageFormat;
use std::fs::File;
use std::io::Cursor;

#[derive(Clone)]
pub enum Cover {
  Extracted(PathBuf),
  NotExtracted,
}

impl Cover {
  pub fn as_path(&self) -> Option<&Path> {
    match self {
      Self::Extracted(path) => Some(path),
      Self::NotExtracted => None,
    }
  }

  pub async fn resize(cover: Vec<u8>, format: ImageFormat, path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref().to_owned();
    let join = spawn_blocking(move || {
      let cursor = Cursor::new(cover);
      let reader = ImageReader::with_format(cursor, format).decode()?;
      let cover = reader.thumbnail(400, 400);

      let file = File::create(&path)?;
      let encoder = WebPEncoder::new_lossless(file);
      cover.write_with_encoder(encoder)?;

      Ok(())
    });

    join.await?
  }

  pub fn path(app: &AppHandle, book_id: i32) -> Result<PathBuf> {
    app
      .path()
      .app_cache_dir()
      .map(|dir| dir.join(format!("covers/{book_id}")))
      .map_err(Into::into)
  }
}

impl From<PathBuf> for Cover {
  fn from(path: PathBuf) -> Self {
    Self::Extracted(path)
  }
}
