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
use tokio::sync::{mpsc, oneshot, Semaphore, SemaphorePermit};
use uuid::Uuid;
use zip::{ZipArchive, ZipWriter};

type TxResult<T> = oneshot::Sender<Result<T>>;

pub type PageMap = OrderedMap<usize, String>;

pub const MAX_FILE_PERMITS: usize = 50;
static FILE_SEMAPHORE: Semaphore = Semaphore::const_new(MAX_FILE_PERMITS);

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
    let (tx, rx) = oneshot::channel();
    let _ = self
      .sender
      .send(Message::GetPages { path, tx })
      .await;

    rx.await?
  }

  pub async fn read_page<P, S>(&self, path: P, page: S) -> Result<Vec<u8>>
  where
    P: AsRef<Path>,
    S: AsRef<str>,
  {
    let path = path.as_ref().to_owned();
    let page = page.as_ref().to_owned();
    let (tx, rx) = oneshot::channel();
    let _ = self
      .sender
      .send(Message::ReadPage { path, page, tx })
      .await;

    rx.await?
  }

  pub async fn delete_page<P, S>(&self, path: P, page: S) -> Result<()>
  where
    P: AsRef<Path>,
    S: AsRef<str>,
  {
    let path = path.as_ref().to_owned();
    let page = page.as_ref().to_owned();
    let (tx, rx) = oneshot::channel();
    let _ = self
      .sender
      .send(Message::DeletePage { path, page, tx })
      .await;

    rx.await?
  }

  pub async fn close(&self, path: impl AsRef<Path>) {
    let path = path.as_ref().to_owned();
    let _ = self.sender.send(Message::Close { path }).await;
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
            || Box::pin(BookFile::open(&path)) as BoxFuture<Result<BookFile>>,
            |book| Box::pin(future::ok(book)),
          )
          .and_then(|it| it.delete_page(&path, page))
          .await;

        let _ = tx.send(result);
      }
      Message::Close { path } => {
        self.books.remove(&path);
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
      .expect("book was just added to the map")
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
      let pages = zip.pages();

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
      let mut file = zip.lock().unwrap();
      let mut page = file.by_name(&page)?;
      let size = usize::try_from(page.size()).unwrap_or_default();
      let mut buf = Vec::with_capacity(size);
      page.read_to_end(&mut buf)?;

      Ok(buf)
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
  fn pages(&self) -> PageMap;
}

impl<T> ZipArchiveExt for ZipArchive<T>
where
  T: Read + Seek,
{
  fn pages(&self) -> PageMap {
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
}
