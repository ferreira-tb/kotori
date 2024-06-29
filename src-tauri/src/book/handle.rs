use crate::book::metadata::Metadata;
use crate::prelude::*;
use crate::utils::collections::OrderedMap;
use crate::utils::glob;
use ahash::{HashMap, HashMapExt};
use futures::future::{self, BoxFuture};
use std::fs::{self, File};
use std::io::{Read, Seek};
use std::sync::{Arc, Mutex};
use std::{fmt, thread};
use strum::Display;
use tokio::sync::{mpsc, oneshot, Notify, Semaphore, SemaphorePermit};
use uuid::Uuid;
use zip::result::{ZipError, ZipResult};
use zip::{ZipArchive, ZipWriter};

type TxResult<T> = oneshot::Sender<Result<T>>;

pub type PageMap = OrderedMap<usize, String>;

pub const MAX_FILE_PERMITS: usize = 50;
static FILE_SEMAPHORE: Semaphore = Semaphore::const_new(MAX_FILE_PERMITS);

/// Sends a message to the actor, awaiting its response with a oneshot channel.
macro_rules! send_tx {
  ($handle:expr, $message:ident { $($item:tt),* }) => {{
    let (tx, rx) = oneshot::channel();
    let _ = $handle.sender.send(Message::$message { tx $(,$item)* }).await;
    rx.await?
  }};
}

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

  pub async fn close(&self, path: impl AsRef<Path>) {
    let path = path.as_ref().to_owned();
    send_notify!(self, Close { path });
  }

  pub async fn get_book_metadata(&self, path: impl AsRef<Path>) -> Result<Option<Metadata>> {
    let path = path.as_ref().to_owned();
    send_tx!(self, Metadata { path })
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
  Close {
    path: PathBuf,
    nt: Arc<Notify>,
  },
  Metadata {
    path: PathBuf,
    tx: TxResult<Option<Metadata>>,
  },
}

pub struct Actor {
  books: HashMap<PathBuf, BookFile>,
  receiver: mpsc::Receiver<Message>,
}

impl Actor {
  fn new(receiver: mpsc::Receiver<Message>) -> Self {
    Self { books: HashMap::new(), receiver }
  }

  pub async fn run(&mut self) {
    while let Some(message) = self.receiver.recv().await {
      debug!(queued_messages = self.receiver.len());
      self.handle_message(message).await;
    }
  }

  async fn handle_message(&mut self, message: Message) {
    trace!(%message, books = self.books.len());
    match message {
      Message::GetPages { path, tx } => {
        let result = self
          .get_book(&path)
          .await
          .map(|it| Arc::clone(&it.pages));

        let _ = tx.send(result);
      }
      Message::ReadPage { path, page, tx } => {
        let result = self
          .get_book(&path)
          .and_then(|it| it.read_page(&page))
          .await;

        let _ = tx.send(result);
      }
      Message::DeletePage { path, page, tx } => {
        let result = self
          .books
          .remove(&path)
          .map_or_else(
            || {
              let future = BookFile::open(&path);
              Box::pin(future) as BoxFuture<Result<BookFile>>
            },
            |book| Box::pin(future::ok(book)),
          )
          .and_then(|it| it.delete_page(&path, page))
          .await;

        let _ = tx.send(result);
      }
      Message::Metadata { path, tx } => {
        let result = self
          .get_book(&path)
          .and_then(BookFile::read_book_metadata)
          .await;

        let _ = tx.send(result);
      }
      Message::Close { path, nt } => {
        self.books.remove(&path);
        nt.notify_one();
      }
    };
  }

  async fn get_book<P>(&mut self, path: P) -> Result<&BookFile>
  where
    P: AsRef<Path>,
  {
    let path = path.as_ref();
    if !self.books.contains_key(path) {
      let book = BookFile::open(&path).await?;
      self.books.insert(path.to_owned(), book);
    }

    self
      .books
      .get(path)
      .map(Ok)
      .expect("book should have been added if it was missing")
  }
}

struct BookFile {
  file: Arc<Mutex<ZipArchive<File>>>,
  pages: Arc<PageMap>,

  #[allow(dead_code)]
  permit: SemaphorePermit<'static>,
}

impl BookFile {
  async fn open(path: impl AsRef<Path>) -> Result<Self> {
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
        permit,
      };

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

  async fn read_book_metadata(&self) -> Result<Option<Metadata>> {
    let zip = Arc::clone(&self.file);
    let join = spawn_blocking(move || {
      zip
        .lock()
        .unwrap()
        .book_metadata()?
        .as_deref()
        .map(serde_json::from_slice)
        .transpose()
        .map_err(Into::into)
    });

    join.await?
  }

  async fn delete_page<P, S>(self, path: P, page: S) -> Result<()>
  where
    P: AsRef<Path>,
    S: AsRef<str>,
  {
    let parent = path.try_parent()?;
    let temp = parent.join(format!("{}.kotori", Uuid::now_v7()));

    let zip = Arc::clone(&self.file);
    let path = path.as_ref().to_owned();
    let page = page.as_ref().to_owned();

    let join = spawn_blocking(move || {
      let mut file = File::create(&temp)?;
      let mut writer = ZipWriter::new(&mut file);

      let mut zip = zip.lock().unwrap();
      let names = zip
        .file_names()
        .filter(|it| *it != page)
        .map(ToOwned::to_owned)
        .collect_vec();

      for name in names {
        let file = zip.by_name(&name)?;
        writer.raw_copy_file(file)?;
      }

      if let Err(err) = writer.finish() {
        fs::remove_file(&temp)?;
        return Err(Into::into(err));
      }

      fs::remove_file(&path)?;
      fs::rename(temp, path)?;

      Ok(())
    });

    join.await?
  }
}

trait ZipArchiveExt {
  fn book_pages(&self) -> PageMap;
  fn book_metadata(&mut self) -> ZipResult<Option<Vec<u8>>>;
  fn read_file(&mut self, name: &str) -> ZipResult<Vec<u8>>;
}

impl<T> ZipArchiveExt for ZipArchive<T>
where
  T: Read + Seek,
{
  fn book_pages(&self) -> PageMap {
    use natord::compare_ignore_case;

    let globset = glob::book_page();
    self
      .file_names()
      .filter(|name| globset.is_match(name))
      .sorted_unstable_by(|a, b| compare_ignore_case(a, b))
      .enumerate()
      .map(|(idx, name)| (idx, name.to_owned()))
      .collect()
  }

  fn book_metadata(&mut self) -> ZipResult<Option<Vec<u8>>> {
    #[cfg(any(debug_assertions, feature = "devtools"))]
    let name = "kotori-dev.json";
    #[cfg(not(any(debug_assertions, feature = "devtools")))]
    let name = "kotori.json";

    match self.read_file(name) {
      Ok(it) => Ok(Some(it)),
      Err(ZipError::FileNotFound) => Ok(None),
      Err(err) => Err(err),
    }
  }

  fn read_file(&mut self, name: &str) -> ZipResult<Vec<u8>> {
    let mut file = self.by_name(name)?;
    let size = usize::try_from(file.size()).unwrap_or_default();
    let mut buf = Vec::with_capacity(size);
    file.read_to_end(&mut buf)?;
    Ok(buf)
  }
}
