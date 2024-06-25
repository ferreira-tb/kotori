use crate::book::{ActiveBook, LibraryBook, MAX_FILE_PERMITS};
use crate::database::entities::book;
use crate::database::prelude::*;
use crate::event::Event;
use crate::prelude::*;
use crate::utils::glob;
use std::sync::Arc;
use tauri_plugin_dialog::{DialogExt, FileDialogBuilder, MessageDialogBuilder, MessageDialogKind};
use tokio::fs;
use tokio::sync::{oneshot, Semaphore};
use tokio::task::JoinSet;
use walkdir::WalkDir;

pub async fn add_folders(app: &AppHandle, folders: &[impl AsRef<Path>]) -> Result<()> {
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
  add_folders(app, &folders).await
}

pub async fn save(app: &AppHandle, path: impl AsRef<Path>) -> Result<book::Model> {
  let path = path.try_string()?;
  let model = book::ActiveModel {
    path: Set(path),
    ..Default::default()
  };

  let kotori = app.kotori();
  let model = Book::insert(model)
    .on_conflict(
      OnConflict::column(book::Column::Path)
        .do_nothing()
        .to_owned(),
    )
    .exec_with_returning(&kotori.db)
    .await?;

  LibraryBook::from_model(app, &model)
    .await
    .and_then(|it| Event::BookAdded(&it).emit(app))?;

  Ok(model)
}

async fn save_many<I>(app: &AppHandle, books: I) -> Result<()>
where
  I: IntoIterator<Item = PathBuf>,
{
  let semaphore = Arc::new(Semaphore::new(MAX_FILE_PERMITS));
  let mut set = books
    .into_iter()
    .map(|path| {
      let app = app.clone();
      let semaphore = Arc::clone(&semaphore);
      async move {
        let _permit = semaphore.acquire_owned().await?;
        save(&app, path).await
      }
    })
    .collect::<JoinSet<_>>();

  let mut models = Vec::with_capacity(set.len());
  while let Some(result) = set.join_next().await {
    if let Ok(model) = result? {
      models.push(model);
    }
  }

  schedule_cover_extraction(app, models);

  Ok(())
}

pub async fn get_all(app: &AppHandle) -> Result<Vec<LibraryBook>> {
  let semaphore = Arc::new(Semaphore::new(MAX_FILE_PERMITS));
  let mut set = Book::get_all(app)
    .await?
    .into_iter()
    .map(|model| {
      let app = app.clone();
      let semaphore = Arc::clone(&semaphore);
      async move {
        let _permit = semaphore.acquire_owned().await.ok()?;

        // Remove the book if the file is missing.
        if let Ok(false) = fs::try_exists(&model.path).await {
          let _ = remove(&app, model.id).await;
          return None;
        }

        let book = LibraryBook::from_model(&app, &model).await.ok()?;
        Some((book, model))
      }
    })
    .collect::<JoinSet<_>>();

  let mut books = Vec::with_capacity(set.len());
  let mut pending = Vec::new();

  while let Some(result) = set.join_next().await {
    if let Some((book, model)) = result? {
      if book.cover.is_none() {
        pending.push(model);
      }

      books.push(book);
    }
  }

  schedule_cover_extraction(app, pending);

  Ok(books)
}

fn schedule_cover_extraction(app: &AppHandle, models: Vec<book::Model>) {
  let semaphore = Arc::new(Semaphore::new(MAX_FILE_PERMITS));
  for model in models {
    let app = app.clone();
    let semaphore = Arc::clone(&semaphore);
    async_runtime::spawn(async move {
      let _permit = semaphore.acquire_owned().await?;
      let book = ActiveBook::from_model(&app, &model)?;
      let path = app.path().cover(model.id)?;
      book.extract_cover(&app, path).await
    });
  }
}

pub async fn remove(app: &AppHandle, id: i32) -> Result<()> {
  let kotori = app.kotori();
  Book::delete_by_id(id).exec(&kotori.db).await?;
  Event::BookRemoved(id).emit(app)?;

  if let Ok(cover) = app.path().cover(id) {
    if fs::try_exists(&cover).await? {
      fs::remove_file(cover).await?;
    }
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

  let path = app.path().cover_dir()?;
  fs::remove_dir_all(path).await.map_err(Into::into)
}
