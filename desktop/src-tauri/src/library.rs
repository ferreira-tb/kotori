use crate::book::{ActiveBook, LibraryBook};
use crate::database::model::{Book, NewFolder};
use crate::event::Event;
use crate::prelude::*;
use crate::utils::glob;
use future_iter::join_set::{IntoJoinSetBy, JoinSetFromIter};
use std::sync::Arc;
use tauri_plugin_dialog::{DialogExt, FileDialogBuilder, MessageDialogBuilder, MessageDialogKind};
use tokio::fs;
use tokio::sync::{oneshot, Semaphore};
use walkdir::WalkDir;

const MAX_FILE_PERMITS: usize = 50;

pub async fn add_folders<I>(app: &AppHandle, folders: I) -> Result<()>
where
  I: IntoIterator<Item = PathBuf>,
{
  let folders = folders.into_iter().collect_vec();
  if folders.is_empty() {
    return Ok(());
  }

  let mut books = Vec::new();
  let library_folders = app.database_handle().get_all_folders().await?;
  let mut current_folders = Vec::with_capacity(folders.len());

  for folder in folders {
    // There's no need to add folders contained in others that have already been saved.
    if library_folders
      .iter()
      .chain(current_folders.iter())
      .any(|it| folder.starts_with(it))
    {
      #[cfg(feature = "tracing")]
      trace!(skip_folder = ?folder);
      continue;
    }

    walk_folder(&mut books, &folder);
    current_folders.push(folder);
  }

  if !current_folders.is_empty() {
    let folders = current_folders
      .into_iter()
      .filter_map(|path| path.try_string().ok())
      .map(|path| NewFolder { path })
      .collect_vec();

    app
      .database_handle()
      .save_folders(folders)
      .await?;
  }

  if !books.is_empty() {
    save_many(app, books).await?;
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
  add_folders(app, folders).await
}

pub async fn save(app: &AppHandle, path: &Path) -> Result<Book> {
  let mut builder = Book::builder(path);
  let book_handle = app.book_handle();
  let cover = book_handle.get_first_page_name(path).await?;
  builder = builder.cover(cover);

  if let Some(metadata) = book_handle.get_metadata(path).await? {
    builder = builder.metadata(metadata);
  }

  let new_book = builder.build(app).await?;
  let model = app.database_handle().save_book(new_book).await?;

  // We could already call `BookHandle::set_metadata` to write the metadata of the saved book,
  // but that doesn't seem like a good idea. After all, the data would only be default values.
  let book = LibraryBook::from_model(app, &model)?;
  Event::BookAdded(&book).emit(app)?;

  book_handle.close(path).await;

  Ok(model)
}

async fn save_many<I>(app: &AppHandle, books: I) -> Result<()>
where
  I: IntoIterator<Item = PathBuf>,
{
  let semaphore = Arc::new(Semaphore::new(MAX_FILE_PERMITS));
  let mut set = books.into_iter().join_set_by(|path| {
    let app = app.clone();
    let semaphore = Arc::clone(&semaphore);
    async move {
      let _permit = semaphore.acquire_owned().await?;
      save(&app, &path).await
    }
  });

  let mut models = Vec::with_capacity(set.len());
  while let Some(result) = set.join_next().await {
    if let Ok(model) = result? {
      models.push(model);
    }
  }

  if !models.is_empty() {
    schedule_cover_extraction(app, models);
  }

  Ok(())
}

pub async fn get_all(app: &AppHandle) -> Result<Vec<LibraryBook>> {
  let mut set = app
    .database_handle()
    .get_all_books()
    .await?
    .into_join_set_by(|model| {
      let app = app.clone();
      async move {
        // Remove the book if the file is missing.
        if let Ok(false) = fs::try_exists(&model.path).await {
          let _ = remove(&app, model.id).await;
          return None;
        }

        let book = LibraryBook::from_model(&app, &model).ok()?;
        Some((book, model))
      }
    });

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

  if !pending.is_empty() {
    schedule_cover_extraction(app, pending);
  }

  Ok(books)
}

fn schedule_cover_extraction<I>(app: &AppHandle, models: I)
where
  I: IntoIterator<Item = Book>,
{
  let permits = MAX_FILE_PERMITS / 5;
  let semaphore = Arc::new(Semaphore::new(permits));
  for model in models {
    let app = app.clone();
    let semaphore = Arc::clone(&semaphore);
    spawn(async move {
      let _permit = semaphore.acquire_owned().await?;
      ActiveBook::from_model(&app, &model)?
        .extract_cover()
        .await
    });
  }
}

pub async fn remove(app: &AppHandle, id: i32) -> Result<()> {
  app.database_handle().remove_book(id).await?;
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

  let title = app.database_handle().get_book_title(id).await?;
  let message = format!("{title} will be removed from the library.");

  MessageDialogBuilder::new(dialog, "Remove book", message)
    .kind(MessageDialogKind::Warning)
    .ok_button_label("Remove")
    .cancel_button_label("Cancel")
    .show(move |response| {
      let _ = tx.send(response);
    });

  if rx.await? {
    remove(app, id).await?;
  }

  Ok(())
}

#[cfg(feature = "devtools")]
pub async fn remove_all(app: &AppHandle) -> Result<()> {
  let handle = app.database_handle();
  handle.remove_all_books().await?;
  handle.remove_all_folders().await?;

  Event::LibraryCleared.emit(app)?;

  let path = app.path().cover_dir()?;
  if let Ok(true) = fs::try_exists(&path).await {
    fs::remove_dir_all(path).await?;
  }

  Ok(())
}

pub async fn scan_book_folders(app: &AppHandle) -> Result<()> {
  let mut books = Vec::new();
  let folders = app.database_handle().get_all_folders().await?;
  for folder in folders {
    walk_folder(&mut books, &folder);
  }

  if !books.is_empty() {
    save_many(app, books).await?;
  }

  Ok(())
}

/// Search recursively for books within the folder.
fn walk_folder(books: &mut Vec<PathBuf>, folder: &Path) {
  let globset = glob::book();
  WalkDir::new(folder)
    .into_iter()
    .flatten()
    .for_each(|entry| {
      let path = entry.into_path();
      if path.is_file() && globset.is_match(&path) {
        books.push(path);
      }
    });
}

#[cfg(feature = "devtools")]
pub async fn add_mock_books(
  app: &AppHandle,
  amount: u8,
  size: usize,
  orientation: crate::image::mock::Orientation,
) -> Result<()> {
  use crate::image::mock::create_book;
  use tokio::task::JoinSet;

  let mut set = JoinSet::new();
  for _ in 0..amount {
    let app = app.clone();
    set.spawn_blocking(move || create_book(&app, size, orientation));
  }

  let mut books = Vec::with_capacity(amount.into());
  while let Some(book) = set.join_next().await {
    books.push(book??);
  }

  if !books.is_empty() {
    let path = app.path().mocks_dir()?.try_string()?;
    app
      .database_handle()
      .save_folders([NewFolder { path }])
      .await?;

    save_many(app, books).await?;
  }

  Ok(())
}
