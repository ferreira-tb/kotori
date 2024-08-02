#[cfg(feature = "devtools")]
pub mod mock;

use crate::prelude::*;
use image::codecs::webp::WebPEncoder;
use image::{ImageFormat, ImageReader};
use std::fs::{self, File};
use std::io::Cursor;

/// Try to guess the image format from the buffer, falling back to the file extension.
pub fn guess_image_format(path: impl AsRef<Path>, buffer: &[u8]) -> Result<ImageFormat> {
  image::guess_format(buffer)
    .or_else(|_| ImageFormat::from_path(path))
    .map_err(Into::into)
}

/// Scale an image down to thumbnail size, writing it to the specified path.
/// This is primarily used to create the cover thumbnails used by the library.
pub fn create_thumbnail(buffer: Vec<u8>, format: ImageFormat, save_as: &Path) -> Result<()> {
  let parent = save_as.try_parent()?;
  fs::create_dir_all(parent)?;

  let cursor = Cursor::new(buffer);
  let reader = ImageReader::with_format(cursor, format).decode()?;
  let thumbnail = reader.thumbnail(400, 400);

  let file = File::create(save_as)?;
  let encoder = WebPEncoder::new_lossless(file);
  thumbnail
    .write_with_encoder(encoder)
    .map_err(Into::into)
}
