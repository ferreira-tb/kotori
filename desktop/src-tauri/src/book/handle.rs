use crate::book::metadata::Metadata;
use crate::prelude::*;
use crate::utils::collections::OrderedMap;
use crate::utils::glob;
use crate::utils::temp::Tempfile;
use ahash::{HashMap, HashMapExt};
use natord::compare_ignore_case;
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::sync::{Arc, Mutex};
use std::{fmt, thread};
use strum::Display;
use tokio::sync::{mpsc, oneshot, Notify, Semaphore, SemaphorePermit};
use zip::result::{ZipError, ZipResult};
use zip::write::SimpleFileOptions as ZipSimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

type TxResult<T> = oneshot::Sender<Result<T>>;

pub(super) type PageMap = OrderedMap<usize, String>;

pub const MAX_FILE_PERMITS: usize = 50;
static FILE_SEMAPHORE: Semaphore = Semaphore::const_new(MAX_FILE_PERMITS);

#[cfg(not(any(debug_assertions, feature = "devtools")))]
const METADATA: &str = "kotori.json";
#[cfg(any(debug_assertions, feature = "devtools"))]
const METADATA: &str = "kotori-dev.json";

/// Send a message to the actor, awaiting its response with a oneshot channel.
macro_rules! send_tx {
  ($handle:expr, $message:ident { $($item:tt),* }) => {{
    let (tx, rx) = oneshot::channel();
    let _ = $handle.sender.send(Message::$message { tx $(,$item)* }).await;
    rx.await?
  }};
}

/// Send a message to the actor, awaiting to be notified by it.
macro_rules! send_notify {
  ($handle:expr, $message:ident { $($item:tt),* }) => {{
    let notify = Arc::new(Notify::new());
    let message = Message::$message { nt: Arc::clone(&notify) $(,$item)* };
    let _ = $handle.sender.send(message).await;
    notify.notified().await;
  }};
}

#[derive(Clone)]
pub struct BookHandle {
  sender: mpsc::Sender<Message>,
}

impl BookHandle {
  pub fn new() -> Self {
    let (sender, receiver) = mpsc::channel(20);
    let mut actor = Actor::new(receiver);

    thread::spawn(move || {
      block_on(async move { actor.run().await });
    });

    Self { sender }
  }

  /// Close the book file, removing it from the cache.
  pub async fn close(&self, path: impl AsRef<Path>) {
    let path = path.as_ref().to_owned();
    send_notify!(self, Close { path });
  }

  pub async fn get_pages(&self, path: impl AsRef<Path>) -> Result<Arc<PageMap>> {
    let path = path.as_ref().to_owned();
    send_tx!(self, GetPages { path })
  }

  pub async fn read_page<P, S>(&self, path: P, page: S) -> Result<Vec<u8>>
  where
    P: AsRef<Path>,
    S: AsRef<str>,
  {
    let path = path.as_ref().to_owned();
    let page = page.as_ref().to_owned();
    send_tx!(self, ReadPage { path, page })
  }

  pub async fn delete_page<P, S>(&self, path: P, page: S) -> Result<()>
  where
    P: AsRef<Path>,
    S: AsRef<str>,
  {
    let path = path.as_ref().to_owned();
    let page = page.as_ref().to_owned();
    send_tx!(self, DeletePage { path, page })
  }

  pub async fn get_metadata(&self, path: impl AsRef<Path>) -> Result<Option<Metadata>> {
    let path = path.as_ref().to_owned();
    let metadata = send_tx!(self, GetMetadata { path })?;

    #[cfg(debug_assertions)]
    if let Some(metadata) = &metadata {
      trace!(get_metadata = ?metadata);
    }

    Ok(metadata)
  }

  pub async fn set_metadata<P>(&self, path: P, metadata: Metadata) -> Result<()>
  where
    P: AsRef<Path>,
  {
    trace!(set_metadata = ?metadata);
    let path = path.as_ref().to_owned();
    send_tx!(self, SetMetadata { path, metadata })
  }

  pub async fn get_first_page_name<P>(&self, path: P) -> Result<String>
  where
    P: AsRef<Path>,
  {
    let path = path.as_ref().to_owned();
    send_tx!(self, GetFirstPageName { path })
  }
}

impl fmt::Debug for BookHandle {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("BookHandle").finish()
  }
}

#[derive(Display)]
#[strum(serialize_all = "snake_case")]
enum Message {
  Close {
    path: PathBuf,
    nt: Arc<Notify>,
  },
  GetPages {
    path: PathBuf,
    tx: TxResult<Arc<PageMap>>,
  },
  ReadPage {
    path: PathBuf,
    page: String,
    tx: TxResult<Vec<u8>>,
  },
  DeletePage {
    path: PathBuf,
    page: String,
    tx: TxResult<()>,
  },
  GetFirstPageName {
    path: PathBuf,
    tx: TxResult<String>,
  },
  GetMetadata {
    path: PathBuf,
    tx: TxResult<Option<Metadata>>,
  },
  SetMetadata {
    path: PathBuf,
    metadata: Metadata,
    tx: TxResult<()>,
  },
}

struct Actor {
  cache: HashMap<PathBuf, BookFile>,
  receiver: mpsc::Receiver<Message>,
}

impl Actor {
  fn new(receiver: mpsc::Receiver<Message>) -> Self {
    Self { cache: HashMap::new(), receiver }
  }

  async fn run(&mut self) {
    while let Some(message) = self.receiver.recv().await {
      trace!(queued_messages = self.receiver.len());
      self.handle_message(message).await;
    }
  }

  async fn handle_message(&mut self, message: Message) {
    trace!(%message, books = self.cache.len());
    match message {
      Message::Close { path, nt } => {
        self.cache.remove(&path);
        nt.notify_one();
      }
      Message::GetPages { path, tx } => {
        let result = self
          .get_book(&path)
          .await
          .map(|it| Arc::clone(&it.pages));

        let _ = tx.send(result);
      }
      Message::ReadPage { path, page, tx } => {
        trace!(read_page = %page);
        let result = self
          .get_book(&path)
          .and_then(|it| it.read_page(&page))
          .await;

        let _ = tx.send(result);
      }
      Message::DeletePage { path, page, tx } => {
        trace!(delete_page = %page);
        let result = self
          .remove_book(&path)
          .and_then(|it| it.delete_page(page))
          .await;

        let _ = tx.send(result);
      }
      Message::GetMetadata { path, tx } => {
        let result = self
          .get_book(&path)
          .and_then(BookFile::read_metadata)
          .await;

        let _ = tx.send(result);
      }
      Message::SetMetadata { path, metadata, tx } => {
        trace!(set_metadata = ?metadata);
        let result = self
          .remove_book(&path)
          .and_then(|it| it.write_metadata(metadata))
          .await;

        let _ = tx.send(result);
      }
      Message::GetFirstPageName { path, tx } => {
        let result = self
          .get_book(&path)
          .await
          .and_then(BookFile::first_page_name);

        let _ = tx.send(result);
      }
    };
  }

  async fn get_book(&mut self, path: &PathBuf) -> Result<&BookFile> {
    if !self.cache.contains_key(path) {
      let book = BookFile::open(&path).await?;
      self.cache.insert(path.clone(), book);
    }

    self
      .cache
      .get(path)
      .map(Ok)
      .expect("book should be in the cache")
  }

  async fn remove_book(&mut self, path: &PathBuf) -> Result<BookFile> {
    if let Some(book) = self.cache.remove(path) {
      Ok(book)
    } else {
      BookFile::open(&path).await
    }
  }
}

struct BookFile {
  file: Arc<Mutex<ZipArchive<File>>>,
  pages: Arc<PageMap>,
  path: PathBuf,

  #[allow(dead_code)]
  permit: SemaphorePermit<'static>,
}

impl BookFile {
  async fn open(path: impl AsRef<Path>) -> Result<Self> {
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

  async fn read_page(&self, page: impl AsRef<str>) -> Result<Vec<u8>> {
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

  async fn read_metadata(&self) -> Result<Option<Metadata>> {
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

  async fn delete_page(self, page: impl AsRef<str>) -> Result<()> {
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

  fn first_page_name(&self) -> Result<String> {
    self
      .pages
      .values()
      .next()
      .map(ToOwned::to_owned)
      .ok_or_else(|| err!(EmptyBook))
  }

  async fn write_metadata(self, metadata: Metadata) -> Result<()> {
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
        .raw_copy_if(&mut writer, |it| *it != METADATA)?;

      writer.start_file(METADATA, ZipSimpleFileOptions::default())?;
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
    match self.read_file(METADATA) {
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
