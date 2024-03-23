mod extractor;
mod page;

use crate::prelude::*;
use crate::utils::{glob, TempDir};
use extractor::Extractor;
use page::Page;
use std::cmp::Ordering;
use walkdir::WalkDir;

#[derive(Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ActiveBook {
  pub(super) path: PathBuf,
  pub(super) title: String,
  pages: Vec<Page>,

  /// Temporary directory where the book is extracted.
  /// It will be automatically deleted when the book is dropped.
  temp_dir: TempDir,

  #[serde(skip_serializing)]
  status: Status,
}

impl ActiveBook {
  pub fn new<P>(path: P) -> Result<Self>
  where
    P: AsRef<Path>,
  {
    let path = path.as_ref();
    let title = path
      .file_stem()
      .ok_or_else(|| Error::InvalidBook(format!("invalid book path: {path:?}")))?
      .to_string_lossy()
      .into_owned()
      .replace('_', " ");

    let book = Self {
      path: path.to_owned(),
      pages: Vec::new(),
      title,
      temp_dir: TempDir::new()?,
      status: Status::default(),
    };

    Ok(book)
  }

  pub fn as_json(&self) -> Result<serde_json::Value> {
    serde_json::to_value(self).map_err(Into::into)
  }

  pub async fn extract(&mut self) -> Result<()> {
    if !matches!(self.status, Status::Extracted) {
      Extractor::new(&self.path)?
        .extract(self.temp_dir.path())
        .await?;

      self.status = Status::Extracted;
      self.update_pages()?;
    }

    Ok(())
  }

  pub async fn extract_cover(&mut self) -> Result<()> {
    if matches!(self.status, Status::NotExtracted) {
      Extractor::new(&self.path)?
        .extract_cover(self.temp_dir.path())
        .await?;

      self.status = Status::OnlyCover;
      self.update_pages()?;
    }

    Ok(())
  }

  fn update_pages(&mut self) -> Result<()> {
    let globset = glob::book_page();
    let pages: Result<Vec<Page>> = WalkDir::new(self.temp_dir.path())
      .into_iter()
      .filter_map(|entry| {
        entry.ok().and_then(|entry| {
          let path = entry.into_path();
          (path.is_file() && globset.is_match(&path)).then(|| Page::try_from(path))
        })
      })
      .collect();

    self.pages = pages?;
    self.pages.shrink_to_fit();
    self.pages.sort_unstable();

    Ok(())
  }
}

impl PartialEq for ActiveBook {
  fn eq(&self, other: &Self) -> bool {
    self.path == other.path
  }
}

impl Eq for ActiveBook {}

impl PartialOrd for ActiveBook {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for ActiveBook {
  fn cmp(&self, other: &Self) -> Ordering {
    natord::compare_ignore_case(&self.title, &other.title)
  }
}

#[derive(Debug, Default)]
enum Status {
  #[default]
  NotExtracted,
  Extracted,
  OnlyCover,
}
