use crate::book::handle::file::BookFile;
use crate::book::handle::message::Message;
use crate::prelude::*;
use ahash::{HashMap, HashMapExt};
use std::sync::Arc;
use tokio::sync::mpsc;

pub(super) struct Actor {
  cache: HashMap<PathBuf, BookFile>,
  receiver: mpsc::Receiver<Message>,
}

impl Actor {
  pub(super) fn new(receiver: mpsc::Receiver<Message>) -> Self {
    Self { cache: HashMap::new(), receiver }
  }

  pub(super) async fn run(&mut self) {
    while let Some(message) = self.receiver.recv().await {
      trace!(queued_messages = self.receiver.len());
      self.handle_message(message).await;
    }
  }

  async fn handle_message(&mut self, message: Message) {
    trace!(%message, actor_cache_size = self.cache.len());
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
