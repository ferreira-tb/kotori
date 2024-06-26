use crate::prelude::*;
use crate::utils::collections::OrderedMap;
use crate::utils::glob;
use natord::compare_ignore_case;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::{fmt, thread};
use strum::Display;
use tokio::sync::{mpsc, oneshot, Semaphore, SemaphorePermit};
use uuid::Uuid;
use zip::{ZipArchive, ZipWriter};

type TxResult<T> = oneshot::Sender<Result<T>>;

pub const MAX_FILE_PERMITS: usize = 50;
static FILE_SEMAPHORE: Semaphore = Semaphore::const_new(MAX_FILE_PERMITS);

#[derive(Clone)]
pub struct BookHandle {
  sender: mpsc::Sender<Message>,
}

impl BookHandle {
  pub fn new(app: &AppHandle) -> Self {
    let (sender, receiver) = mpsc::channel(20);
    let mut actor = Actor::new(app, receiver);

    thread::spawn(move || {
      async_runtime::block_on(async move { actor.run().await });
    });

    Self { sender }
  }

  pub async fn get_pages(&self, path: impl AsRef<Path>) -> Result<OrderedMap<usize, String>> {
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
    tx: TxResult<OrderedMap<usize, String>>,
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
  app: AppHandle,
  books: HashMap<PathBuf, BookFile>,
  receiver: mpsc::Receiver<Message>,
}

impl Actor {
  fn new(app: &AppHandle, receiver: mpsc::Receiver<Message>) -> Self {
    Self {
      app: app.clone(),
      books: HashMap::new(),
      receiver,
    }
  }

  pub async fn run(&mut self) {
    while let Some(message) = self.receiver.recv().await {
      debug!(queued_messages = self.receiver.len());
      self
        .handle_message(message)
        .await
        .into_log(&self.app);
    }
  }

  async fn handle_message(&mut self, message: Message) -> Result<()> {
    trace!(%message, books = self.books.len());
    match message {
      Message::GetPages { path, tx } => {
        let result = self
          .get_book(&path)
          .await
          .map(|it| &it.pages)
          .cloned();

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
          .get_book(&path)
          .and_then(|it| it.delete_page(&path, &page))
          .await;

        let _ = tx.send(result);
      }
      Message::Close { path } => {
        self.books.remove(&path);
      }
    };

    Ok(())
  }

  async fn get_book(&mut self, path: impl AsRef<Path>) -> Result<&BookFile> {
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
  pages: OrderedMap<usize, String>,

  #[allow(dead_code)]
  permit: SemaphorePermit<'static>,
}

impl BookFile {
  async fn open(path: impl AsRef<Path>) -> Result<Self> {
    debug!(available_file_permits = FILE_SEMAPHORE.available_permits());
    let permit = FILE_SEMAPHORE.acquire().await?;

    let path = path.as_ref().to_owned();
    let join = async_runtime::spawn_blocking(move || {
      let reader = File::open(&path)?;
      let zip = ZipArchive::new(reader)?;

      let globset = glob::book_page();
      let pages = zip
        .file_names()
        .filter(|name| globset.is_match(name))
        .sorted_unstable_by(|a, b| compare_ignore_case(a, b))
        .enumerate()
        .map(|(idx, name)| (idx, name.to_owned()))
        .collect();

      let file = BookFile {
        file: Arc::new(Mutex::new(zip)),
        pages,
        permit,
      };

      Ok(file)
    });

    join.await?
  }

  async fn read_page(&self, page: impl AsRef<str>) -> Result<Vec<u8>> {
    let zip = Arc::clone(&self.file);
    let page = page.as_ref().to_owned();

    let join = async_runtime::spawn_blocking(move || {
      let mut file = zip.lock().unwrap();
      let mut page = file.by_name(&page)?;
      let size = usize::try_from(page.size()).unwrap_or_default();
      let mut buf = Vec::with_capacity(size);
      page.read_to_end(&mut buf)?;

      Ok(buf)
    });

    join.await?
  }

  async fn delete_page<P, S>(&self, path: P, page: S) -> Result<()>
  where
    P: AsRef<Path>,
    S: AsRef<str>,
  {
    let parent = path.try_parent()?;
    let temp = parent.join(format!("{}.kotori", Uuid::now_v7()));

    let zip = Arc::clone(&self.file);
    let path = path.as_ref().to_owned();
    let page = page.as_ref().to_owned();

    let join = async_runtime::spawn_blocking(move || {
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

      fs::rename(temp, path)?;

      Ok(())
    });

    join.await?
  }
}
