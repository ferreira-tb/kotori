use super::actor::{Actor, Status};
use super::message::Message;
use crate::prelude::*;
use std::sync::atomic::{self, AtomicUsize};
use std::sync::mpsc;
use std::{fmt, thread};
use tokio::sync::oneshot;

static WORKER_ID: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone)]
pub(super) struct Worker {
  id: usize,
  sender: mpsc::Sender<Message>,
}

impl Worker {
  pub(super) fn new(app: &AppHandle) -> Self {
    let id = WORKER_ID.fetch_add(1, atomic::Ordering::Relaxed);
    let (sender, receiver) = mpsc::channel();

    app
      .run_on_main_thread(move || {
        let mut actor = Actor::new(receiver);
        let name = format!("scheduler-worker-{id}");

        #[cfg(feature = "tracing")]
        info!("spawning worker thread: {name}");

        thread::Builder::new()
          .name(name)
          .spawn(move || actor.run())
          .expect("failed to spawn worker thread");
      })
      .expect("failed to spawn from main thread");

    Self { id, sender }
  }

  /// Check if the actor managed by this worker has a file open.
  pub(super) async fn has_file(&self, path: &Path) -> bool {
    let (tx, rx) = oneshot::channel();
    self.send(Message::HasFile { tx, path: path.to_owned() });
    rx.await.unwrap_or(false)
  }

  pub(super) async fn status(&self) -> Status {
    let (tx, rx) = oneshot::channel();
    self.send(Message::Status { tx });
    rx.await.unwrap_or_default()
  }

  pub(super) fn send(&self, message: Message) {
    #[cfg(feature = "tracing")]
    trace!("sending {:?} to worker {}", message, self.id);

    let _ = self.sender.send(message);
  }
}

impl fmt::Debug for Worker {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Worker")
      .field("id", &self.id)
      .finish_non_exhaustive()
  }
}

impl PartialEq for Worker {
  fn eq(&self, other: &Self) -> bool {
    self.id == other.id
  }
}

impl Eq for Worker {}
