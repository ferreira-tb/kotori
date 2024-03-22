mod book;

use crate::events::Event;
use crate::prelude::*;
use crate::utils::glob;
use book::ActiveBook;
use tauri_plugin_dialog::{DialogExt, FileDialogBuilder};
use walkdir::WalkDir;

pub struct Library {
  books: Vec<ActiveBook>,
}

impl Library {
  pub fn new() -> Self {
    Self { books: Vec::new() }
  }

  pub async fn add<M, R>(app: &M) -> Result<()>
  where
    R: Runtime,
    M: Manager<R>,
  {
    let dialog = app.dialog().clone();
    let response = FileDialogBuilder::new(dialog).blocking_pick_folders();

    if let Some(paths) = response {
      let globset = glob::book();
      let mut books = Vec::new();

      for path in paths {
        let entries: Result<Vec<ActiveBook>> = WalkDir::new(&path)
          .into_iter()
          .filter_map(|entry| {
            entry.ok().and_then(|entry| {
              let path = entry.into_path();
              (path.is_file() && globset.is_match(&path)).then(|| ActiveBook::new(path))
            })
          })
          .collect();

        books.extend(entries?);
      }
    }

    Ok(())
  }

  pub fn find_active<P>(&self, path: P) -> Option<&ActiveBook>
  where
    P: AsRef<Path>,
  {
    self
      .books
      .iter()
      .find(|b| b.metadata.path == path.as_ref())
  }

  pub fn find_active_mut<P>(&mut self, path: P) -> Option<&mut ActiveBook>
  where
    P: AsRef<Path>,
  {
    self
      .books
      .iter_mut()
      .find(|b| b.metadata.path == path.as_ref())
  }

  pub fn insert_active(&mut self, book: ActiveBook) {
    self.books.push(book);
  }

  pub async fn open_book<M, R>(app: &M) -> Result<()>
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
      let mut library = state.library.lock().await;

      if let Some(book) = library.find_active_mut(&response.path) {
        book.extract().await?;

        let json = book.as_json()?;
        app.emit(Event::BookOpened.as_str(), json)?;

        return Ok(());
      }

      let mut book = ActiveBook::new(&response.path)?;
      book.extract().await?;

      let json = book.as_json()?;
      app.emit(Event::BookOpened.as_str(), json)?;

      library.insert_active(book);
    }

    Ok(())
  }
}
