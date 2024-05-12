use crate::prelude::*;
use crate::utils::collections::OrderedMap;
use crate::utils::glob;
use natord::compare_ignore_case;
use std::fs::File;
use std::io::Read;
use tempfile::NamedTempFile;
use tokio::fs;
use zip::{ZipArchive, ZipWriter};

#[derive(Clone)]
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

  pub async fn has_page(&self, name: &str) -> bool {
    let handle = self.handle.lock().await;
    let mut file_names = handle.file_names();
    file_names.any(|it| it == name)
  }

  pub async fn get_page_by_name(&self, name: &str) -> Result<Vec<u8>> {
    let mut handle = self.handle.lock().await;
    let mut file = handle.by_name(name)?;

    let size = usize::try_from(file.size()).unwrap_or_default();
    let mut buf = Vec::with_capacity(size);
    file.read_to_end(&mut buf)?;

    Ok(buf)
  }

  pub async fn delete_page_by_name(&self, path: impl AsRef<Path>, name: &str) -> Result<()> {
    let name = name.to_owned();
    let handle = Arc::clone(&self.handle);

    let join = spawn_blocking(move || {
      let mut temp = NamedTempFile::new()?;
      let mut writer = ZipWriter::new(&mut temp);

      // We shouldn't skip files that aren't pages here.
      let mut handle = block_on(handle.lock());
      let names = handle
        .file_names()
        .filter(|it| *it != name)
        .map(ToOwned::to_owned)
        .collect_vec();

      for name in names {
        let file = handle.by_name(&name)?;
        writer.raw_copy_file(file)?;
      }

      if let Err(err) = writer.finish() {
        temp.into_temp_path().close()?;
        return Err(err);
      }

      Ok(temp)
    });

    let temp = join.await??;
    fs::remove_file(&path).await?;
    temp.into_temp_path().persist(path).ok();

    Ok(())
  }
}
