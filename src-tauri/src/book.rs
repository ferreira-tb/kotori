mod extractor;
mod page;

use crate::error::{Error, Result};
use crate::utils::{img_globset, Json, TempDir};
use crate::State;
use extractor::Extractor;
use page::Page;
use serde::Serialize;
use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::api::path::app_cache_dir;
use tauri::Config;
use walkdir::WalkDir;

#[derive(Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Book {
  /// Original path of the book file.
  pub path: PathBuf,
  /// Temporary directory where the book is extracted.
  /// It will be automatically deleted when the book is dropped.
  temp_dir: TempDir,

  title: String,
  pages: Vec<Page>,

  #[serde(skip_serializing)]
  status: Status,
}

impl Book {
  pub async fn new<P>(path: P, config: &Config, state: &State<'_>) -> Result<Self>
  where
    P: AsRef<Path>,
  {
    let path = path.as_ref();
    let books = state.books.lock().await;
    if books.iter().any(|b| b.path == path) {
      return Err(Error::AlreadyExists);
    }

    drop(books);

    let cache = Self::book_cache(config)?;
    let temp_dir = TempDir::try_from(cache.as_path())?;

    let title = path
      .file_stem()
      .ok_or_else(|| Error::InvalidBook("file stem not found".to_string()))?
      .to_string_lossy()
      .into_owned()
      .replace('_', " ");

    let book = Self {
      path: path.to_owned(),
      temp_dir,
      title,
      pages: Vec::default(),
      status: Status::default(),
    };

    Ok(book)
  }

  pub fn as_json(&self) -> Result<Json> {
    serde_json::to_value(self).map_err(Into::into)
  }

  fn book_cache(config: &Config) -> Result<PathBuf> {
    let dir = app_cache_dir(config)
      .map(|dir| dir.join("books"))
      .ok_or_else(|| Error::CacheNotFound)?;

    if let Ok(false) = dir.try_exists() {
      fs::create_dir_all(&dir)?;
    }

    Ok(dir)
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
    let globset = img_globset()?;
    let pages: Result<Vec<Page>> = WalkDir::new(self.temp_dir.path())
      .into_iter()
      .filter_map(|entry| {
        entry.ok().and_then(|entry| {
          let path = entry.into_path();
          if globset.is_match(&path) {
            Some(Page::try_from(path))
          } else {
            None
          }
        })
      })
      .collect();

    self.pages = pages?;
    self.pages.shrink_to_fit();
    self.pages.sort_unstable();

    Ok(())
  }
}

impl PartialEq for Book {
  fn eq(&self, other: &Self) -> bool {
    self.path == other.path
  }
}

impl Eq for Book {}

impl PartialOrd for Book {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for Book {
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
