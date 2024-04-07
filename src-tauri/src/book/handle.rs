use crate::prelude::*;
use crate::utils::{glob, OrderedMap};
use natord::compare_ignore_case;
use std::fs::File;
use std::io::Read;
use tauri::async_runtime::spawn_blocking;
use zip::ZipArchive;

pub(super) struct Handle {
  handle: Arc<Mutex<ZipArchive<File>>>,
}

impl Handle {
  pub async fn new(path: impl AsRef<Path>) -> Result<Self> {
    let path = path.as_ref().to_owned();
    let join: JoinResult<ZipArchive<File>> = spawn_blocking(move || {
      let file = File::open(path)?;
      ZipArchive::new(file).map_err(Into::into)
    });

    join.await?.map(|zip| Self {
      handle: Arc::new(Mutex::new(zip)),
    })
  }

  pub async fn pages(&self) -> OrderedMap<usize, String> {
    let globset = glob::book_page();
    let handle = self.handle.lock().await;
    handle
      .file_names()
      .filter(|name| globset.is_match(name))
      .sorted_unstable_by(|a, b| compare_ignore_case(a, b))
      .enumerate()
      .map(|(idx, name)| (idx, name.to_owned()))
      .collect()
  }

  pub async fn by_name(&self, name: &str) -> Result<Vec<u8>> {
    let mut handle = self.handle.lock().await;
    let mut file = handle.by_name(name)?;

    let size = usize::try_from(file.size()).unwrap_or_default();
    let mut buf = Vec::with_capacity(size);
    file.read_to_end(&mut buf)?;

    Ok(buf)
  }
}
