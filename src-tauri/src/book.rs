mod extractor;
mod page;

use crate::error::{Error, Result};
use crate::utils::{img_globset, TempDir};
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
  temp_dir: TempDir,

  title: String,
  pages: Vec<Page>,
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
    self.pages.clear();

    let globset = img_globset()?;
    let entries = WalkDir::new(self.temp_dir.path())
      .into_iter()
      .filter_map(|entry| {
        entry.ok().and_then(|entry| {
          let path = entry.path();
          if entry.file_type().is_file() && globset.is_match(path) {
            Some(path.to_path_buf())
          } else {
            None
          }
        })
      });

    for entry in entries {
      let page = Page::new(entry)?;
      self.pages.push(page);
    }

    self.pages.shrink_to_fit();
    self.pages.sort_unstable();

    Ok(())
  }

  pub async fn open(&mut self) -> Result<()> {
    self.extract().await?;

    println!("open book: {}", self.title);

    Ok(())
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

#[derive(Debug, Default, Serialize)]
enum Status {
  #[default]
  NotExtracted,
  Extracted,
  OnlyCover,
}
