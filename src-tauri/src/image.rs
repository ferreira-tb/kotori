use std::fs::File;
use std::io::{Cursor, Write};

use image::codecs::webp::WebPEncoder;
use image::io::Reader as ImageReader;
use image::{ImageFormat, Rgb, RgbImage};

use crate::prelude::*;

#[cfg(any(debug_assertions, feature = "devtools"))]
#[derive(Copy, Clone, Debug)]
pub enum Orientation {
  Landscape,
  Portrait,
}

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

#[cfg(any(debug_assertions, feature = "devtools"))]
pub async fn create_mock_book(
  app: &AppHandle,
  size: usize,
  orientation: Orientation,
) -> Result<PathBuf> {
  use rand::Rng;
  use uuid::Uuid;
  use zip::write::SimpleFileOptions;
  use zip::ZipWriter;

  let path = app
    .path()
    .dev_cache_dir()
    .map(|it| it.join("mocks"))?;

  tokio::fs::create_dir_all(&path).await?;

  let name = format!("{}.zip", Uuid::new_v4());
  let path = path.join(name);

  let join = spawn_blocking(move || {
    let mut rng = rand::thread_rng();
    let mut file = File::create(&path)?;
    let mut writer = ZipWriter::new(&mut file);

    let (width, height) = match orientation {
      Orientation::Landscape => (1280, 520),
      Orientation::Portrait => (760, 1200),
    };

    for _ in 0..size {
      let mut buf = RgbImage::new(width, height);
      let rgb = Rgb([rng.gen(), rng.gen(), rng.gen()]);
      for pixel in buf.pixels_mut() {
        *pixel = rgb;
      }

      let mut image = Vec::with_capacity(buf.len());
      let encoder = WebPEncoder::new_lossless(&mut image);
      buf.write_with_encoder(encoder)?;

      let name = format!("{}.webp", Uuid::now_v7());
      writer.start_file(name, SimpleFileOptions::default())?;
      writer.write_all(&image)?;
    }

    if let Err(err) = writer.finish() {
      std::fs::remove_file(&path)?;
      return Err(Into::into(err));
    }

    Ok(path)
  });

  join.await?
}
