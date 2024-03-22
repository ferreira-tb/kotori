use crate::prelude::*;
use crate::utils::img_globset;
use std::fs::File;
use std::io::Read;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use tokio::task::JoinSet;
use zip::ZipArchive;

type Zip = Arc<Mutex<ZipArchive<File>>>;

pub struct Extractor {
  zip: Zip,
  files: Vec<String>,
}

impl Extractor {
  pub fn new<P>(path: P) -> Result<Self>
  where
    P: AsRef<Path>,
  {
    let zip = File::open(path.as_ref())?;
    let zip = ZipArchive::new(zip)?;

    let globset = img_globset()?;
    let files: Vec<String> = zip
      .file_names()
      .filter(|n| globset.is_match(n))
      .map(ToOwned::to_owned)
      .collect();

    if files.is_empty() {
      return Err(Error::Empty);
    }

    let extractor = Self {
      zip: Arc::new(Mutex::new(zip)),
      files,
    };

    Ok(extractor)
  }

  pub async fn extract<P>(self, directory: P) -> Result<()>
  where
    P: AsRef<Path>,
  {
    if self.files.is_empty() {
      return Err(Error::Empty);
    }

    let directory = directory.as_ref();
    let mut set = JoinSet::new();

    for file_name in self.files {
      let zip = Arc::clone(&self.zip);
      let directory = directory.to_owned();
      set.spawn(async move { Extractor::extract_file(zip, file_name, directory).await });
    }

    while let Some(result) = set.join_next().await {
      result.map_err(|e| anyhow::anyhow!(e))??;
    }

    Ok(())
  }

  pub async fn extract_cover<P>(mut self, directory: P) -> Result<()>
  where
    P: AsRef<Path>,
  {
    self
      .files
      .sort_unstable_by(|a, b| natord::compare_ignore_case(a, b));

    let cover = self.files.first().ok_or_else(|| Error::Empty)?;
    let zip = Arc::clone(&self.zip);
    Extractor::extract_file(zip, cover, &directory).await?;

    Ok(())
  }

  async fn extract_file<P, F>(zip: Zip, file_name: F, directory: P) -> Result<()>
  where
    F: AsRef<str>,
    P: AsRef<Path>,
  {
    let file_name = file_name.as_ref();
    let buf = {
      let mut zip = zip.lock().await;
      let mut file = zip.by_name(file_name)?;

      let mut buf = Vec::new();
      file.read_to_end(&mut buf)?;

      buf
    };

    let file_name = Path::new(file_name)
      .file_name()
      .and_then(|name| name.to_str());

    if let Some(file_name) = file_name {
      let path = directory.as_ref().join(file_name);
      let mut file = tokio::fs::File::create(&path).await?;
      file.write_all(&buf).await?;
    }

    Ok(())
  }
}
