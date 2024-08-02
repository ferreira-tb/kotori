use crate::prelude::*;
use image::codecs::webp::WebPEncoder;
use image::{Rgb, RgbImage};
use rand::Rng;
use std::fs::{self, File};
use std::io::Write;
use uuid::Uuid;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

#[derive(Copy, Clone, Debug)]
pub enum Orientation {
  Landscape,
  Portrait,
}

#[cfg_attr(feature = "tracing", instrument(skip(app), level = "trace"))]
pub fn create_book(app: &AppHandle, size: usize, orientation: Orientation) -> Result<PathBuf> {
  #[cfg(feature = "tracing")]
  let start = Instant::now();

  let path = app.path().mocks_dir();
  fs::create_dir_all(&path)?;

  let name = format!("{}.zip", Uuid::new_v4());
  let path = path.join(name);
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
    fs::remove_file(&path)?;
    return Err(Into::into(err));
  }

  #[cfg(feature = "tracing")]
  trace!("mock book created in {:?}", start.elapsed());

  Ok(path)
}
