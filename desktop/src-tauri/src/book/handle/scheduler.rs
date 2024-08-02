use super::message::Message;
use super::worker::Worker;
use crate::prelude::*;
use ahash::{HashMap, HashMapExt};
use std::fmt;
use std::num::NonZero;
use std::thread::available_parallelism;
use tokio::sync::{Mutex, Semaphore, SemaphorePermit};

static FILE_PERMITS: Semaphore = Semaphore::const_new(100);

pub(super) struct Scheduler {
  app: AppHandle,
  files: Mutex<HashMap<PathBuf, FileGuard>>,
  workers: Mutex<Vec<Worker>>,
  max_workers: usize,
}

impl Scheduler {
  pub(super) fn new(app: &AppHandle) -> Self {
    Self {
      app: app.clone(),
      files: Mutex::new(HashMap::new()),
      workers: Mutex::new(Vec::new()),
      max_workers: available_parallelism().map_or(1, NonZero::get),
    }
  }

  pub(super) async fn schedule(&self, message: Message) -> Result<()> {
    let Some(path) = message.path() else {
      return Ok(());
    };

    #[cfg(feature = "tracing")]
    trace!("scheduling {message:?} for {path:?}");

    let mut files = self.files.lock().await;
    if let Some(file) = files.get(&path) {
      if file.worker.has_file(&path).await {
        file.worker.send(message);
        return Ok(());
      }

      files.remove(&path);
    }

    drop(files);

    let worker = self.get_available_worker().await;
    let file = FileGuard::create(worker).await;
    file.worker.send(message);

    self.files.lock().await.insert(path, file);

    Ok(())
  }

  async fn get_available_worker(&self) -> Worker {
    use super::actor::Status;

    let mut workers = self.workers.lock().await;
    let mut busy = Vec::with_capacity(workers.len());
    for worker in workers.iter() {
      match worker.status().await {
        Status::Idle => return worker.clone(),
        Status::Busy(amount) => {
          busy.push((worker, amount));
        }
      }
    }

    if workers.len() < self.max_workers {
      let worker = Worker::new(&self.app);
      workers.push(worker.clone());
      worker
    } else {
      busy
        .into_iter()
        .min_by_key(|(_, amount)| *amount)
        .map(|(worker, _)| worker.clone())
        .expect("there should be at least one worker")
    }
  }
}

impl fmt::Debug for Scheduler {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Scheduler")
      .field("max_workers", &self.max_workers)
      .finish_non_exhaustive()
  }
}

struct FileGuard {
  worker: Worker,
  _permit: SemaphorePermit<'static>,
}

impl FileGuard {
  pub async fn create(worker: Worker) -> Self {
    let permit = FILE_PERMITS
      .acquire()
      .await
      .expect("semaphore will never close");

    Self { worker, _permit: permit }
  }
}

impl fmt::Debug for FileGuard {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("FileGuard")
      .field("worker", &self.worker)
      .finish_non_exhaustive()
  }
}
