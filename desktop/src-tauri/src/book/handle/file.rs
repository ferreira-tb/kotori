use super::PageMap;
use crate::book::metadata::Metadata;
use crate::prelude::*;
use crate::utils::glob;
use crate::utils::temp::Tempfile;
use natord::compare_ignore_case;
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::sync::{Arc, Mutex};
use tokio::sync::{Semaphore, SemaphorePermit};
use zip::result::{ZipError, ZipResult};
use zip::write::SimpleFileOptions as ZipSimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

pub const MAX_FILE_PERMITS: usize = 50;
static FILE_SEMAPHORE: Semaphore = Semaphore::const_new(MAX_FILE_PERMITS);

#[cfg(not(any(debug_assertions, feature = "devtools")))]
const METADATA_FILENAME: &str = "kotori.json";
#[cfg(any(debug_assertions, feature = "devtools"))]
const METADATA_FILENAME: &str = "kotori-dev.json";

pub(super) struct BookFile {
  file: Arc<Mutex<ZipArchive<File>>>,
  path: PathBuf,
  pub(super) pages: Arc<PageMap>,

  #[allow(dead_code)]
  permit: SemaphorePermit<'static>,
}

impl BookFile {
  pub(super) async fn open(path: impl AsRef<Path>) -> Result<Self> {
    #[cfg(any(debug_assertions, feature = "devtools"))]
    let start = std::time::Instant::now();

    debug!(available_file_permits = FILE_SEMAPHORE.available_permits());
    let permit = FILE_SEMAPHORE.acquire().await?;

    let path = path.as_ref().to_owned();
    let join = spawn_blocking(move || {
      let reader = File::open(&path)?;
      let zip = ZipArchive::new(reader)?;
      let pages = zip.book_pages();

      let file = BookFile {
        file: Arc::new(Mutex::new(zip)),
        pages: Arc::new(pages),
        path,
        permit,
      };

      #[cfg(any(debug_assertions, feature = "devtools"))]
      info!("book opened in {:?}", start.elapsed());

      Ok(file)
    });

    join.await?
  }

  pub(super) async fn read_page(&self, page: impl AsRef<str>) -> Result<Vec<u8>> {
    let zip = Arc::clone(&self.file);
    let page = page.as_ref().to_owned();

    let join = spawn_blocking(move || {
      zip
        .lock()
        .unwrap()
        .read_file(&page)
        .map_err(Into::into)
    });

    join.await?
  }

  pub(super) async fn read_metadata(&self) -> Result<Option<Metadata>> {
    let zip = Arc::clone(&self.file);
    let join = spawn_blocking(move || {
      zip
        .lock()
        .unwrap()
        .read_book_metadata()?
        .as_deref()
        .map(serde_json::from_slice)
        .transpose()
        .map_err(Into::into)
    });

    join.await?
  }

  pub(super) async fn delete_page(self, page: impl AsRef<str>) -> Result<()> {
    #[cfg(any(debug_assertions, feature = "devtools"))]
    let start = std::time::Instant::now();

    let page = page.as_ref().to_owned();
    let join = spawn_blocking(move || {
      let parent = self.path.try_parent()?;
      let mut temp = Tempfile::new_in(parent)?;
      let mut writer = ZipWriter::new(&mut temp.file);

      self
        .file
        .lock()
        .unwrap()
        .raw_copy_if(&mut writer, |it| *it != page)?;

      writer.finish()?;
      std::fs::remove_file(&self.path)?;
      std::fs::rename(&temp.path, self.path)?;

      #[cfg(any(debug_assertions, feature = "devtools"))]
      info!("page deleted in {:?}", start.elapsed());

      Ok(())
    });

    join.await?
  }

  pub(super) fn first_page_name(&self) -> Result<String> {
    self
      .pages
      .values()
      .next()
      .map(ToOwned::to_owned)
      .ok_or_else(|| err!(EmptyBook))
  }

  pub(super) async fn write_metadata(self, metadata: Metadata) -> Result<()> {
    #[cfg(any(debug_assertions, feature = "devtools"))]
    let start = std::time::Instant::now();

    let join = spawn_blocking(move || {
      let parent = self.path.try_parent()?;
      let mut temp = Tempfile::new_in(parent)?;
      let mut writer = ZipWriter::new(&mut temp.file);

      self
        .file
        .lock()
        .unwrap()
        .raw_copy_if(&mut writer, |it| *it != METADATA_FILENAME)?;

      writer.start_file(METADATA_FILENAME, ZipSimpleFileOptions::default())?;
      serde_json::to_writer_pretty(&mut writer, &metadata)?;

      writer.finish()?;
      std::fs::remove_file(&self.path)?;
      std::fs::rename(&temp.path, self.path)?;

      #[cfg(any(debug_assertions, feature = "devtools"))]
      info!("metadata written in {:?}", start.elapsed());

      Ok(())
    });

    join.await?
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
