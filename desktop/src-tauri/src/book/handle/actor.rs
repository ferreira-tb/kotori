use super::file::BookFile;
use super::message::Message;
use crate::prelude::*;
use ahash::{HashMap, HashMapExt};
use std::fmt;
use std::sync::mpsc;
use strum::EnumIs;

pub(super) struct Actor {
  cache: HashMap<PathBuf, BookFile>,
  receiver: mpsc::Receiver<Message>,
}

impl Actor {
  pub(super) fn new(receiver: mpsc::Receiver<Message>) -> Self {
    Self { cache: HashMap::new(), receiver }
  }

  pub(super) fn run(&mut self) {
    while let Ok(message) = self.receiver.recv() {
      self.handle_message(message);
    }
  }

  #[cfg_attr(feature = "tracing", instrument)]
  fn handle_message(&mut self, message: Message) {
    #[cfg(feature = "tracing")]
    trace!(actor_cache_size = self.cache.len(), ?message);

    match message {
      Message::Close { path, nt } => {
        self.cache.remove(&path);
        nt.notify_one();
      }
      Message::DeletePage { path, page, tx } => {
        let result = self
          .get_book_owned(&path)
          .and_then(|it| it.delete_page(&page));

        let _ = tx.send(result);
      }
      Message::ExtractCover { path, page, save_as, tx } => {
        let result = self
          .get_book_mut(&path)
          .and_then(|it| it.extract_cover(&page, &save_as));

        let _ = tx.send(result);
      }
      Message::GetFirstPageName { path, tx } => {
        let result = self
          .get_book(&path)
          .and_then(BookFile::first_page_name);

        let _ = tx.send(result);
      }
      Message::GetMetadata { path, tx } => {
        let result = self
          .get_book_mut(&path)
          .and_then(BookFile::read_metadata);

        let _ = tx.send(result);
      }
      Message::GetPages { path, tx } => {
        let result = self.get_book(&path).map(BookFile::pages);
        let _ = tx.send(result);
      }
      Message::HasBookFileInCache { path, tx } => {
        let _ = tx.send(self.cache.contains_key(&path));
      }
      Message::ReadPage { path, page, tx } => {
        let result = self
          .get_book_mut(&path)
          .and_then(|it| it.read_page(&page));

        let _ = tx.send(result);
      }
      Message::SetMetadata { path, metadata, tx } => {
        let result = self
          .get_book_owned(&path)
          .and_then(|it| it.write_metadata(&metadata));

        let _ = tx.send(result);
      }
      Message::Status { tx } => {
        let _ = tx.send(self.status());
      }
    };
  }

  fn ensure_cache_contains(&mut self, path: &Path) -> Result<()> {
    if !self.cache.contains_key(path) {
      let book = BookFile::open(path)?;
      self.cache.insert(path.to_path_buf(), book);
    }

    Ok(())
  }

  fn get_book(&mut self, path: &Path) -> Result<&BookFile> {
    self.ensure_cache_contains(path)?;
    self
      .cache
      .get(path)
      .map(Ok)
      .expect("book should be in the cache")
  }

  fn get_book_mut(&mut self, path: &Path) -> Result<&mut BookFile> {
    self.ensure_cache_contains(path)?;
    self
      .cache
      .get_mut(path)
      .map(Ok)
      .expect("book should be in the cache")
  }

  fn get_book_owned(&mut self, path: &Path) -> Result<BookFile> {
    if let Some(book) = self.cache.remove(path) {
      Ok(book)
    } else {
      BookFile::open(path)
    }
  }

  fn status(&self) -> Status {
    if self.cache.is_empty() {
      Status::Idle
    } else {
      Status::Busy(self.cache.len())
    }
  }
}

impl fmt::Debug for Actor {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Actor")
      .field("cache", &self.cache.len())
      .finish_non_exhaustive()
  }
}

#[derive(Debug, Default, EnumIs)]
pub(super) enum Status {
  #[default]
  Idle,
  Busy(usize),
}
