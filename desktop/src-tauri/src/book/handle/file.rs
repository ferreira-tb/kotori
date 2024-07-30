use super::PageMap;
use crate::book::metadata::Metadata;
use crate::prelude::*;
use crate::utils::glob;
use crate::utils::temp::Tempfile;
use natord::compare_ignore_case;
use std::fs::{self, File};
use std::io::{Read, Seek, Write};
use std::sync::Arc;
use zip::result::{ZipError, ZipResult};
use zip::write::SimpleFileOptions as ZipSimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

#[cfg(not(any(debug_assertions, feature = "devtools")))]
const METADATA_FILENAME: &str = "kotori.json";
#[cfg(any(debug_assertions, feature = "devtools"))]
const METADATA_FILENAME: &str = "kotori-dev.json";

pub(super) struct BookFile {
  file: ZipArchive<File>,
  path: PathBuf,
  pub(super) pages: Arc<PageMap>,
}

impl BookFile {
  pub(super) fn open<P>(path: P) -> Result<Self>
  where
    P: AsRef<Path>,
  {
    let path = path.as_ref().to_owned();

    #[cfg(feature = "tracing")]
    let start = std::time::Instant::now();
    #[cfg(feature = "tracing")]
    tracing::info!("opening book at {}", path.display());

    let reader = File::open(&path)?;
    let zip = ZipArchive::new(reader)?;
    let pages = zip.book_pages();

    let file = BookFile {
      file: zip,
      pages: Arc::new(pages),
      path,
    };

    #[cfg(feature = "tracing")]
    tracing::info!("book opened in {:?}", start.elapsed());

    Ok(file)
  }

  pub(super) fn read_page(&mut self, page: &str) -> Result<Vec<u8>> {
    self.file.read_file(page).map_err(Into::into)
  }

  pub(super) fn read_metadata(&mut self) -> Result<Option<Metadata>> {
    self
      .file
      .read_book_metadata()?
      .as_deref()
      .map(serde_json::from_slice)
      .transpose()
      .map_err(Into::into)
  }

  pub(super) fn delete_page(mut self, page: &str) -> Result<()> {
    #[cfg(feature = "tracing")]
    let start = std::time::Instant::now();

    let parent = self.path.try_parent()?;
    let mut temp = Tempfile::new_in(parent)?;
    let mut writer = ZipWriter::new(&mut temp.file);

    self
      .file
      .raw_copy_if(&mut writer, |it| *it != page)?;

    writer.finish()?;
    fs::remove_file(&self.path)?;
    fs::rename(&temp.path, self.path)?;

    #[cfg(feature = "tracing")]
    tracing::info!("page deleted in {:?}", start.elapsed());

    Ok(())
  }

  pub(super) fn first_page_name(&self) -> Result<String> {
    self
      .pages
      .values()
      .next()
      .map(ToOwned::to_owned)
      .ok_or_else(|| err!(EmptyBook))
  }

  pub(super) fn write_metadata(mut self, metadata: &Metadata) -> Result<()> {
    #[cfg(feature = "tracing")]
    let start = std::time::Instant::now();

    let parent = self.path.try_parent()?;
    let mut temp = Tempfile::new_in(parent)?;
    let mut writer = ZipWriter::new(&mut temp.file);

    self
      .file
      .raw_copy_if(&mut writer, |it| *it != METADATA_FILENAME)?;

    writer.start_file(METADATA_FILENAME, ZipSimpleFileOptions::default())?;
    serde_json::to_writer_pretty(&mut writer, &metadata)?;

    writer.finish()?;
    fs::remove_file(&self.path)?;
    fs::rename(&temp.path, self.path)?;

    #[cfg(feature = "tracing")]
    tracing::info!("metadata written in {:?}", start.elapsed());

    Ok(())
  }
}

trait ZipArchiveExt {
  fn book_pages(&self) -> PageMap;

  fn file_names_by<F>(&mut self, f: F) -> Vec<String>
  where
    F: FnMut(&&str) -> bool;

  fn raw_copy_if<W, F>(&mut self, writer: &mut ZipWriter<&mut W>, f: F) -> ZipResult<()>
  where
    W: Write + Seek,
    F: FnMut(&&str) -> bool;

  fn read_book_metadata(&mut self) -> ZipResult<Option<Vec<u8>>>;
  fn read_file(&mut self, name: &str) -> ZipResult<Vec<u8>>;
}

impl<T> ZipArchiveExt for ZipArchive<T>
where
  T: Read + Seek,
{
  fn book_pages(&self) -> PageMap {
    let globset = glob::book_page();
    self
      .file_names()
      .filter(|name| globset.is_match(name))
      .sorted_unstable_by(|a, b| compare_ignore_case(a, b))
      .enumerate()
      .map(|(idx, name)| (idx, name.to_owned()))
      .collect()
  }

  fn file_names_by<F>(&mut self, f: F) -> Vec<String>
  where
    F: FnMut(&&str) -> bool,
  {
    self.file_names().filter(f).map_into().collect()
  }

  fn read_file(&mut self, name: &str) -> ZipResult<Vec<u8>> {
    let mut file = self.by_name(name)?;
    let size = usize::try_from(file.size()).unwrap_or_default();
    let mut buf = Vec::with_capacity(size);
    file.read_to_end(&mut buf)?;
    Ok(buf)
  }

  fn read_book_metadata(&mut self) -> ZipResult<Option<Vec<u8>>> {
    match self.read_file(METADATA_FILENAME) {
      Ok(it) => Ok(Some(it)),
      Err(ZipError::FileNotFound) => Ok(None),
      Err(err) => Err(err),
    }
  }

  fn raw_copy_if<W, F>(&mut self, writer: &mut ZipWriter<&mut W>, f: F) -> ZipResult<()>
  where
    W: Write + Seek,
    F: FnMut(&&str) -> bool,
  {
    for name in self.file_names_by(f) {
      let file = self.by_name(&name)?;
      writer.raw_copy_file(file)?;
    }

    Ok(())
  }
}
