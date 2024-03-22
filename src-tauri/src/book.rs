mod extractor;
mod page;

use crate::error::{Error, Result};
use crate::utils::{img_globset, Event, Json, TempDir};
use crate::Kotori;
use extractor::Extractor;
use page::Page;
use serde::Serialize;
use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{Manager, Runtime};
use tauri_plugin_dialog::{DialogExt, FileDialogBuilder};
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
  pub async fn new<M, R, P>(app: &M, path: P) -> Result<Self>
  where
    R: Runtime,
    M: Manager<R>,
    P: AsRef<Path>,
  {
    let cache = Self::book_cache(app)?;
    let temp_dir = TempDir::try_from(cache.as_path())?;

    let path = path.as_ref();
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

  fn book_cache<M, R>(app: &M) -> Result<PathBuf>
  where
    R: Runtime,
    M: Manager<R>,
  {
    let dir = app.path().app_cache_dir()?.join("books");
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

  pub async fn open<M, R>(app: &M) -> Result<()>
  where
    R: Runtime,
    M: Manager<R>,
  {
    let dialog = app.dialog().clone();
    let response = FileDialogBuilder::new(dialog)
      .add_filter("Book", &["cbr", "cbz"])
      .blocking_pick_file();

    if let Some(response) = response {
      let state = app.state::<Kotori>();
      let mut books = state.books.lock().await;

      if books.iter().any(|b| b.path == response.path) {
        let book = books
          .iter_mut()
          .find(|b| b.path == response.path)
          .expect("book should exist");

        book.extract().await?;
        
        let json = book.as_json()?;
        app.emit(Event::BookOpened.as_str(), json)?;

        return Ok(());
      }

      let mut book = Book::new(app, &response.path).await?;
      book.extract().await?;

      let json = book.as_json()?;
      app.emit(Event::BookOpened.as_str(), json)?;

      books.push(book);
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
