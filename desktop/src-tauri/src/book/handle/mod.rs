mod actor;
mod file;
mod message;

use crate::book::metadata::Metadata;
use crate::prelude::*;
use crate::utils::collections::OrderedMap;
use actor::Actor;
pub use file::MAX_FILE_PERMITS;
use message::Message;
use std::fmt;
use std::sync::Arc;
use tokio::sync::{mpsc, Notify};

pub(super) type PageMap = OrderedMap<usize, String>;

/// Send a message to the actor, awaiting its response with a oneshot channel.
macro_rules! send_tx {
  ($handle:expr, $message:ident { $($item:tt),* }) => {{
    let (tx, rx) = tokio::sync::oneshot::channel();
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
    
    spawn(async move { actor.run().await });

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
