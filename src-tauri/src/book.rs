mod extractor;
mod page;

use crate::error::{Error, Result};
use crate::utils::img_globset;
use crate::State;
use extractor::Extractor;
use page::Page;
use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::api::path::app_cache_dir;
use tauri::Config;
use tempfile::{tempdir_in, TempDir};
use walkdir::WalkDir;

#[derive(Debug)]
pub struct Book {
  /// Original path of the book file.
  pub path: PathBuf,
  /// Temporary directory where the book is extracted.
  temp_dir: TempDir,

  title: String,
  pages: Vec<Page>,
  extract_status: extractor::Status,
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
    let temp_dir = tempdir_in(cache)?;

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
      extract_status: extractor::Status::default(),
    };

    Ok(book)
  }

  pub async fn extract(&mut self) -> Result<()> {
    if !matches!(self.extract_status, extractor::Status::Extracted) {
      let extractor = Extractor::new(&self.path)?;
      extractor.extract(&self.temp_dir).await?;

      self.extract_status = extractor::Status::Extracted;
      self.update_pages()?;
    }

    Ok(())
  }

  pub async fn extract_cover(&mut self) -> Result<()> {
    if matches!(self.extract_status, extractor::Status::NotExtracted) {
      let extractor = Extractor::new(&self.path)?;
      extractor.extract_cover(&self.temp_dir).await?;

      self.extract_status = extractor::Status::OnlyCover;
      self.update_pages()?;
    }

    Ok(())
  }

  fn update_pages(&mut self) -> Result<()> {
    self.pages.clear();

    let globset = img_globset()?;
    let entries = WalkDir::new(&self.temp_dir)
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

  #[must_use]
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
