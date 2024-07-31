#[cfg(feature = "devtools")]
pub mod mock;

use crate::prelude::*;
use image::codecs::webp::WebPEncoder;
use image::{ImageFormat, ImageReader};
use std::fs::{self, File};
use std::io::Cursor;

/// Scales an image down to thumbnail size, writing it to `path`.
/// This is primarily used to create the cover thumbnails used by the library.
#[cfg_attr(feature = "tracing", instrument(skip(buf)))]
pub fn create_thumbnail(buf: Vec<u8>, format: ImageFormat, path: &Path) -> Result<()> {
  #[cfg(feature = "tracing")]
  let start = std::time::Instant::now();

  let parent = path.try_parent()?;
  fs::create_dir_all(parent)?;

  let cursor = Cursor::new(buf);
  let reader = ImageReader::with_format(cursor, format).decode()?;
  let thumbnail = reader.thumbnail(400, 400);

  let file = File::create(path)?;
  let encoder = WebPEncoder::new_lossless(file);
  thumbnail.write_with_encoder(encoder)?;

  #[cfg(feature = "tracing")]
  info!("thumbnail created in {:?}", start.elapsed());

  Ok(())
}
