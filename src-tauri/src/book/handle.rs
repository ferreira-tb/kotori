use crate::prelude::*;
use crate::utils::collections::OrderedMap;
use crate::utils::glob;
use natord::compare_ignore_case;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::{fmt, thread};
use strum::Display;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;
use zip::{ZipArchive, ZipWriter};

type TxResult<T> = oneshot::Sender<Result<T>>;

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
      self
        .handle_message(message)
        .await
        .into_log(&self.app);
    }
  }

  async fn handle_message(&mut self, message: Message) -> Result<()> {
    #[cfg(debug_assertions)]
    {
      let books = self
        .books
        .keys()
        .filter_map(|it| it.try_str().ok())
        .collect_vec();

      debug!(%message, ?books);
    }

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
          .await
          .and_then(|it| it.read_page(&page));

        let _ = tx.send(result);
      }
      Message::DeletePage { path, page, tx } => {
        let result = self
          .get_book(&path)
          .await
          .and_then(|it| it.delete_page(path, &page));

        let _ = tx.send(result);
      }
      Message::Close { path } => {
        self.books.remove(&path);
      }
    };

    Ok(())
  }

  async fn get_book(&mut self, path: impl AsRef<Path>) -> Result<&mut BookFile> {
    let path = path.as_ref();
    if !self.books.contains_key(path) {
      let book = BookFile::open(&path).await?;
      self.books.insert(path.to_owned(), book);
    }

    self.books.get_mut(path).map(Ok).unwrap()
  }
}

#[derive(Clone)]
pub struct BookHandle {
  sender: mpsc::Sender<Message>,
}

impl BookHandle {
  pub fn new(app: &AppHandle) -> Self {
    let (sender, receiver) = mpsc::channel(8);
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

struct BookFile {
  file: ZipArchive<File>,
  pages: OrderedMap<usize, String>,
}

impl BookFile {
  async fn open(path: impl AsRef<Path>) -> Result<Self> {
    let path = path.as_ref().to_owned();
    let join = async_runtime::spawn_blocking(move || {
      let mut file = BookFile {
        file: ZipArchive::new(File::open(&path)?)?,
        pages: OrderedMap::default(),
      };

      file.get_pages();

      Ok(file)
    });

    join.await?
  }

  fn get_pages(&mut self) {
    let globset = glob::book_page();
    self.pages = self
      .file
      .file_names()
      .filter(|name| globset.is_match(name))
      .sorted_unstable_by(|a, b| compare_ignore_case(a, b))
      .enumerate()
      .map(|(idx, name)| (idx, name.to_owned()))
      .collect();
  }

  fn read_page(&mut self, page: &str) -> Result<Vec<u8>> {
    let mut page = self.file.by_name(page)?;
    let size = usize::try_from(page.size()).unwrap_or_default();
    let mut buf = Vec::with_capacity(size);
    page.read_to_end(&mut buf)?;

    Ok(buf)
  }

  fn delete_page(&mut self, path: impl AsRef<Path>, page: &str) -> Result<()> {
    let parent = path.try_parent()?;
    let temp = parent.join(format!("{}.kotori", Uuid::now_v7()));

    let mut file = File::create(&temp)?;
    let mut writer = ZipWriter::new(&mut file);

    let names = self
      .file
      .file_names()
      .filter(|it| *it != page)
      .map(ToOwned::to_owned)
      .collect_vec();

    for name in names {
      let file = self.file.by_name(&name)?;
      writer.raw_copy_file(file)?;
    }

    if let Err(err) = writer.finish() {
      std::fs::remove_file(&temp)?;
      return Err(Into::into(err));
    }

    std::fs::rename(temp, path)?;

    Ok(())
  }
}
