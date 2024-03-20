mod page;

use crate::error::{Error, Result};
use crate::utils::img_glob;
use crate::State;
use globset::GlobSetBuilder;
use page::Page;
use std::cmp::Ordering;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use tauri::api::path::app_cache_dir;
use tauri::Config;
use tempfile::{tempdir_in, TempDir};
use walkdir::WalkDir;
use zip::ZipArchive;

#[derive(Debug)]
pub struct Book {
  /// Original path of the book file.
  pub path: PathBuf,
  /// Temporary directory where the book is extracted.
  temp_dir: TempDir,

  title: String,
  pages: Vec<Page>,
  extract_status: ExtractStatus,
}

impl Book {
  pub fn new<P>(path: P, config: &Config, state: &State<'_>) -> Result<Self>
  where
    P: AsRef<Path>,
  {
    let path = path.as_ref();
    let books = state.books.lock().unwrap();
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
      extract_status: ExtractStatus::default(),
    };

    Ok(book)
  }

  pub fn extract(&mut self) -> Result<()> {
    if matches!(self.extract_status, ExtractStatus::Extracted) {
      return Ok(());
    }

    let bytes = fs::read(&self.path)?;
    let cursor = Cursor::new(bytes);

    ZipArchive::new(cursor)
      .unwrap()
      .extract(&self.temp_dir)
      .unwrap();

    self.extract_status = ExtractStatus::Extracted;

    self.update_pages()?;

    Ok(())
  }

  fn update_pages(&mut self) -> Result<()> {
    let globset = GlobSetBuilder::new()
      .add(img_glob("*.jpg").unwrap())
      .add(img_glob("*.jpeg").unwrap())
      .add(img_glob("*.png").unwrap())
      .add(img_glob("*.gif").unwrap())
      .add(img_glob("*.webp").unwrap())
      .build()
      .unwrap();

    self.pages.clear();

    for entry in WalkDir::new(&self.temp_dir) {
      if !matches!(entry, Ok(ref entry) if entry.file_type().is_file()) {
        continue;
      }

      let path = entry
        .expect("entry already checked")
        .path()
        .to_path_buf();

      if !globset.is_match(&path) {
        continue;
      }

      let page = Page::new(path)?;
      self.pages.push(page);
    }

    self.pages.shrink_to_fit();
    self.pages.sort_unstable();

    Ok(())
  }

  pub fn open(&mut self) -> Result<()> {
    self.extract()?;

    println!("open book: {}", self.title);
    for page in &self.pages {
      println!("  page: {}", page.path.display());
    }

    println!("\n\n");

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

#[derive(Debug)]
enum ExtractStatus {
  Extracted,
  NotExtracted,
  // OnlyCover,
}

impl Default for ExtractStatus {
  fn default() -> Self {
    ExtractStatus::NotExtracted
  }
}
