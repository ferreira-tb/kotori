use crate::book::{cover, ActiveBook, LibraryBook};
use crate::database::entities::book;
use crate::database::prelude::*;
use crate::event::Event;
use crate::prelude::*;
use crate::utils::glob;
use tauri_plugin_dialog::{DialogExt, FileDialogBuilder, MessageDialogBuilder, MessageDialogKind};
use tokio::{fs, sync::oneshot, task::JoinSet};
use walkdir::WalkDir;

pub async fn add(app: &AppHandle, folders: &[impl AsRef<Path>]) -> Result<()> {
  if !folders.is_empty() {
    let globset = glob::book();
    let mut books = Vec::new();

    for folder in folders {
      for entry in WalkDir::new(folder).into_iter().flatten() {
        let path = entry.into_path();
        if path.is_file() && globset.is_match(&path) {
          books.push(path);
        }
      }
    }

    if !books.is_empty() {
      save_many(app, books).await?;
    }
  }

  Ok(())
}

pub async fn add_with_dialog(app: &AppHandle) -> Result<()> {
  let (tx, rx) = oneshot::channel();
  let dialog = app.dialog().clone();

  FileDialogBuilder::new(dialog).pick_folders(move |response| {
    let _ = tx.send(response.unwrap_or_default());
  });

  let folders = rx.await?;
  add(app, &folders).await
}

pub async fn save(app: AppHandle, path: impl AsRef<Path>) -> Result<()> {
  let path = path.try_string()?;
  let model = book::ActiveModel {
    path: Set(path),
    ..Default::default()
  };

  let kotori = app.kotori();
  let book = Book::insert(model)
    .on_conflict(
      OnConflict::column(book::Column::Path)
        .do_nothing()
        .to_owned(),
    )
    .exec_with_returning(&kotori.db)
    .await?;

  let payload = LibraryBook::from_model(&app, &book).await?;
  Event::BookAdded(&payload).emit(&app)?;

  let active_book = ActiveBook::from_model(&app, &book)?;
  let cover = cover::path(&app, book.id)?;
  active_book.extract_cover(&app, cover).await?;

  Ok(())
}

async fn save_many<I>(app: &AppHandle, books: I) -> Result<()>
where
  I: IntoIterator<Item = PathBuf>,
{
  let mut tasks = books
    .into_iter()
    .map(|path| save(app.clone(), path))
    .collect::<JoinSet<_>>();

  while let Some(result) = tasks.join_next().await {
    result?.into_log(app);
  }

  Ok(())
}

pub async fn get_all(app: &AppHandle) -> Result<Vec<LibraryBook>> {
  let mut set = Book::get_all(app)
    .await?
    .into_iter()
    .map(|model| to_library_book(app.clone(), model))
    .collect::<JoinSet<_>>();

  let mut books = Vec::with_capacity(set.len());
  while let Some(book) = set.join_next().await {
    match book {
      Ok(Some(book)) => books.push(book),
      Err(error) => warn!(%error),
      _ => {}
    }
  }

  Ok(books)
}

async fn to_library_book(app: AppHandle, model: book::Model) -> Option<LibraryBook> {
  // Remove the book if the file is missing.
  if let Ok(false) = fs::try_exists(&model.path).await {
    remove(&app, model.id).await.into_log(&app);
    return None;
  }

  let book = LibraryBook::from_model(&app, &model).await;
  if matches!(book, Ok(ref it) if it.cover.is_none()) {
    let result: Result<()> = try {
      let book = ActiveBook::from_model(&app, &model)?;
      let path = cover::path(&app, model.id)?;
      book.extract_cover(&app, path).await?;
    };

    result.into_log(&app);
  }

  book.ok()
}

pub async fn remove(app: &AppHandle, id: i32) -> Result<()> {
  let kotori = app.kotori();
  Book::delete_by_id(id).exec(&kotori.db).await?;
  Event::BookRemoved(id).emit(app)?;

  if let Ok(cover) = cover::path(app, id) {
    fs::remove_file(cover).await?;
  }

  Ok(())
}

pub async fn remove_with_dialog(app: &AppHandle, id: i32) -> Result<()> {
  let (tx, rx) = oneshot::channel();
  let dialog = app.dialog().clone();

  let title = Book::get_title(app, id).await?;
  let message = format!("{title} will be removed from the library.");

  MessageDialogBuilder::new(dialog, "Remove book", message)
    .kind(MessageDialogKind::Warning)
    .ok_button_label("Remove")
    .cancel_button_label("Cancel")
    .show(move |response| {
      let _ = tx.send(response);
    });

  if let Ok(true) = rx.await {
    remove(app, id).await?;
  }

  Ok(())
}

pub async fn remove_all(app: &AppHandle) -> Result<()> {
  use sea_query::Query;

  let kotori = app.kotori();
  let builder = kotori.db.get_database_backend();

  let stmt = Query::delete().from_table(Book).to_owned();
  kotori.db.execute(builder.build(&stmt)).await?;
  Event::LibraryCleared.emit(app)?;

  let path = cover::base_path(app)?;
  fs::remove_dir_all(path).await.map_err(Into::into)
}
