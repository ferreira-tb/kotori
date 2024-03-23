mod book;

use crate::event::Event;
use crate::prelude::*;
use crate::utils::{date, glob};
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

  pub async fn add_with_dialog<M, R>(app: &M) -> Result<()>
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
        let entries: Vec<ActiveBook> = WalkDir::new(&path)
          .into_iter()
          .filter_map(|entry| {
            entry.ok().and_then(|entry| {
              let path = entry.into_path();
              (path.is_file() && globset.is_match(&path))
                .then(|| ActiveBook::new(path).ok())
                .flatten()
            })
          })
          .collect();

        books.extend(entries);
      }

      if !books.is_empty() {
        Library::save_books(app, &books).await?;
      }
    }

    Ok(())
  }

  pub fn find_active_mut<P>(&mut self, path: P) -> Option<&mut ActiveBook>
  where
    P: AsRef<Path>,
  {
    self
      .books
      .iter_mut()
      .find(|b| b.path == path.as_ref())
  }

  pub fn insert_active(&mut self, book: ActiveBook) {
    self.books.push(book);
  }

  async fn save_books<M, R>(app: &M, books: &[ActiveBook]) -> Result<()>
  where
    R: Runtime,
    M: Manager<R>,
  {
    use crate::database::prelude::*;

    let as_model = |book: &ActiveBook| -> Option<book::ActiveModel> {
      let path = book.path.to_str().map(ToString::to_string)?;
      let model = book::ActiveModel {
        id: NotSet,
        path: Set(path),
        created_at: Set(date::now()),
        updated_at: Set(date::now()),
        ..Default::default()
      };

      Some(model)
    };

    let database = app.state::<Database>();
    let books: Vec<book::ActiveModel> = books.iter().filter_map(as_model).collect();

    if books.is_empty() {
      return Ok(());
    }

    let on_conflict = OnConflict::column(book::Column::Path)
      .do_nothing()
      .to_owned();

    Book::insert_many(books)
      .on_conflict(on_conflict)
      .do_nothing()
      .exec(&database.conn)
      .await?;

    Ok(())
  }

  pub async fn open_with_dialog<M, R>(app: &M) -> Result<()>
  where
    R: Runtime,
    M: Manager<R>,
  {
    let dialog = app.dialog().clone();
    let response = FileDialogBuilder::new(dialog)
      .add_filter("Book", &["cbr", "cbz"])
      .blocking_pick_file();

    if let Some(response) = response {
      let kotori = app.state::<Kotori>();
      let mut library = kotori.library.lock().await;

      if let Some(book) = library.find_active_mut(&response.path) {
        book.extract().await?;

        let json = book.as_json()?;
        app.emit(Event::OpenBook.as_str(), json)?;

        return Ok(());
      }

      // It may take a while to extract the book, so we drop the lock.
      drop(library);

      let mut book = ActiveBook::new(&response.path)?;
      book.extract().await?;

      let json = book.as_json()?;
      app.emit(Event::OpenBook.as_str(), json)?;

      let mut library = kotori.library.lock().await;
      library.insert_active(book);
    }

    Ok(())
  }
}
