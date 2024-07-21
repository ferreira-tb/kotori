#[cfg(any(debug_assertions, feature = "devtools"))]
pub mod mock;

use crate::prelude::*;
use image::codecs::webp::WebPEncoder;
use image::io::Reader as ImageReader;
use image::ImageFormat;
use std::fs::File;
use std::io::Cursor;

/// Scales an image down to thumbnail size, writing it to `path`.
/// This is primarily use to create the cover thumbnails used by the library.
pub async fn create_thumbnail<P>(buf: Vec<u8>, format: ImageFormat, path: P) -> Result<()>
where
  P: AsRef<Path>,
{
  let path = path.as_ref().to_owned();
  let parent = path.try_parent()?;
  tokio::fs::create_dir_all(parent).await?;

  let join = spawn_blocking(move || {
    let cursor = Cursor::new(buf);
    let reader = ImageReader::with_format(cursor, format).decode()?;
    let thumbnail = reader.thumbnail(400, 400);

    let file = File::create(&path)?;
    let encoder = WebPEncoder::new_lossless(file);
    thumbnail.write_with_encoder(encoder)?;

    Ok(())
  });

  join.await?
}
