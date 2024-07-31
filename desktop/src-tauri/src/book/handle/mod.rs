mod actor;
mod file;
mod message;

use crate::book::metadata::Metadata;
use crate::prelude::*;
use crate::utils::collections::OrderedMap;
use crate::{send_notify, send_tx};
use actor::Actor;
use message::Message;
use std::sync::{mpsc, Arc};
use std::{fmt, thread};

pub(super) type PageMap = OrderedMap<usize, String>;

#[derive(Clone)]
pub struct BookHandle {
  sender: mpsc::Sender<Message>,
}

impl BookHandle {
  pub fn new() -> Self {
    let (sender, receiver) = mpsc::channel();
    let mut actor = Actor::new(receiver);

    thread::spawn(move || actor.run());

    Self { sender }
  }

  /// Close the book file, removing it from the cache.
  pub async fn close(&self, path: &Path) {
    let path = path.to_owned();
    send_notify!(self, Close { path });
  }

  pub async fn get_pages(&self, path: &Path) -> Result<Arc<PageMap>> {
    let path = path.to_owned();
    send_tx!(self, GetPages { path })
  }

  pub async fn read_page(&self, path: &Path, page: &str) -> Result<Vec<u8>> {
    let path = path.to_owned();
    let page = page.to_owned();
    send_tx!(self, ReadPage { path, page })
  }

  pub async fn delete_page(&self, path: &Path, page: &str) -> Result<()> {
    let path = path.to_owned();
    let page = page.to_owned();
    send_tx!(self, DeletePage { path, page })
  }

  pub async fn get_metadata(&self, path: &Path) -> Result<Option<Metadata>> {
    let path = path.to_owned();
    let metadata = send_tx!(self, GetMetadata { path })?;

    #[cfg(feature = "tracing")]
    if let Some(metadata) = &metadata {
      trace!(get_metadata = ?metadata);
    }

    Ok(metadata)
  }

  pub async fn set_metadata(&self, path: &Path, metadata: Metadata) -> Result<()> {
    #[cfg(feature = "tracing")]
    trace!(set_metadata = ?metadata);

    let path = path.to_owned();
    send_tx!(self, SetMetadata { path, metadata })
  }

  pub async fn get_first_page_name(&self, path: &Path) -> Result<String> {
    let path = path.to_owned();
    send_tx!(self, GetFirstPageName { path })
  }
}

impl fmt::Debug for BookHandle {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("BookHandle").finish()
  }
}
