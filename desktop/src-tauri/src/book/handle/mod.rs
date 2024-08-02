mod actor;
mod file;
mod message;
mod scheduler;
mod worker;

use super::metadata::Metadata;
use crate::prelude::*;
use crate::utils::collections::OrderedMap;
use message::Message;
use scheduler::Scheduler;
use std::fmt;
use std::sync::Arc;
use tokio::sync::{oneshot, Notify};

pub(super) type PageMap = OrderedMap<usize, String>;

macro_rules! schedule_tx {
  ($handle:expr, $message:ident { $($item:tt),* }) => {{
    let (tx, rx) = oneshot::channel();
    $handle.scheduler.schedule(Message::$message { tx $(,$item)* }).await?;
    rx.await?
  }};
}

macro_rules! schedule_notify {
  ($handle:expr, $message:ident { $($item:tt),* }) => {{
    let notify = Arc::new(Notify::new());
    let message = Message::$message { nt: Arc::clone(&notify) $(,$item)* };
    $handle.scheduler.schedule(message).await?;
    notify.notified().await;
    Ok(())
  }};
}

#[derive(Clone)]
pub struct BookHandle {
  scheduler: Arc<Scheduler>,
}

impl BookHandle {
  pub fn new(app: &AppHandle) -> Self {
    Self {
      scheduler: Arc::new(Scheduler::new(app)),
    }
  }

  /// Close the book file, removing it from the cache.
  pub async fn close(&self, path: &Path) -> Result<()> {
    let path = path.to_owned();
    schedule_notify!(self, Close { path })
  }

  pub async fn get_pages(&self, path: &Path) -> Result<Arc<PageMap>> {
    let path = path.to_owned();
    schedule_tx!(self, GetPages { path })
  }

  pub async fn read_page(&self, path: &Path, page: &str) -> Result<Vec<u8>> {
    let path = path.to_owned();
    let page = page.to_owned();
    schedule_tx!(self, ReadPage { path, page })
  }

  pub async fn delete_page(&self, path: &Path, page: &str) -> Result<()> {
    let path = path.to_owned();
    let page = page.to_owned();
    schedule_tx!(self, DeletePage { path, page })
  }

  pub async fn get_metadata(&self, path: &Path) -> Result<Option<Metadata>> {
    let path = path.to_owned();
    schedule_tx!(self, GetMetadata { path })
  }

  pub async fn set_metadata(&self, path: &Path, metadata: Metadata) -> Result<()> {
    let path = path.to_owned();
    schedule_tx!(self, SetMetadata { path, metadata })
  }

  pub async fn get_first_page_name(&self, path: &Path) -> Result<String> {
    let path = path.to_owned();
    schedule_tx!(self, GetFirstPageName { path })
  }
}

impl fmt::Debug for BookHandle {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("BookHandle").finish()
  }
}
