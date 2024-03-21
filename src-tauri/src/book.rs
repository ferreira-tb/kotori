mod page;

use crate::error::{Error, Result};
use crate::utils::img_globset;
use crate::State;
use page::Page;
use std::cmp::Ordering;
use std::fs::{self, File};
use std::io::{Read, Write};
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

    let zip = File::open(&self.path)?;
    let mut zip = ZipArchive::new(zip).unwrap();

    let globset = img_globset()?;
    let file_names: Vec<String> = zip
      .file_names()
      .filter_map(|n| globset.is_match(n).then(|| n.to_owned()))
      .collect();

    for file_name in file_names {
      let mut file = zip.by_name(&file_name)?;
      let mut buf = Vec::new();
      file.read_to_end(&mut buf)?;

      let file_name = Path::new(&file_name)
        .file_name()
        .and_then(|name| name.to_str());

      if let Some(file_name) = file_name {
        let path = self.temp_dir.path().join(file_name);
        let mut file = File::create(&path)?;
        file.write_all(&buf)?;
      }
    }

    self.extract_status = ExtractStatus::Extracted;

    self.update_pages()?;

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

  pub fn open(&mut self) -> Result<()> {
    self.extract()?;

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
