use std::sync::Arc;

use kotori_entity::book;
use kotori_entity::prelude::{Book, Folder};
use tauri_plugin_dialog::{DialogExt, FileDialogBuilder, MessageDialogBuilder, MessageDialogKind};
use tokio::fs;
use tokio::sync::{oneshot, Semaphore};
use walkdir::WalkDir;

use crate::book::{ActiveBook, LibraryBook, MAX_FILE_PERMITS};
use crate::database::{BookExt, FolderExt};
use crate::event::Event;
#[cfg(any(debug_assertions, feature = "devtools"))]
use crate::image::Orientation;
use crate::prelude::*;
use crate::utils::glob;

pub async fn add_folders<F>(app: &AppHandle, folders: &[F]) -> Result<()>
where
  F: AsRef<Path>,
{
  if folders.is_empty() {
    return Ok(());
  }

  let mut books = Vec::new();
  let library_folders = Folder::get_all(app).await?;
  let mut current_folders = Vec::with_capacity(folders.len());

  for folder in folders {
    let folder = folder.as_ref();

    // There's no need to add folders contained in others that have already been saved.
    if library_folders
      .iter()
      .chain(current_folders.iter())
      .any(|it| folder.starts_with(it))
    {
      trace!(skip_folder = ?folder);
      continue;
    }

    current_folders.push(folder.to_owned());
    walk_folder(&mut books, folder);
  }

  if !current_folders.is_empty() {
    Folder::create_many(app, current_folders).await?;
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
  add_folders(app, &folders).await
}

pub async fn save<P>(app: &AppHandle, path: P) -> Result<Option<book::Model>>
where
  P: AsRef<Path>,
{
  if Book::has_path(app, &path).await? {
    return Ok(None);
  }

  let mut builder = Book::builder(&path);

  let book_handle = app.book_handle();
  let cover = book_handle.get_first_page_name(&path).await?;
  builder = builder.cover(cover);

  if let Some(metadata) = book_handle.get_metadata(&path).await? {
    builder = builder.metadata(metadata);
  }

  // This will be `None` if an unique constraint violation occurs.
  let model = builder.build(app).await?;

  // We could already call `BookHandle::set_metadata` to write the metadata of the saved book,
  // but that doesn't seem like a good idea. After all, the data would only be default values.
  if let Some(model) = &model {
    let book = LibraryBook::from_model(app, model)?;
    Event::BookAdded(&book).emit(app)?;
  }

  book_handle.close(path).await;

  Ok(model)
}

async fn save_many<I>(app: &AppHandle, books: I) -> Result<()>
where
  I: IntoIterator<Item = PathBuf>,
{
  let semaphore = Arc::new(Semaphore::new(MAX_FILE_PERMITS));
  let mut set = books.into_iter().into_join_set_by(|path| {
    let app = app.clone();
    let semaphore = Arc::clone(&semaphore);
    async move {
      let _permit = semaphore.acquire_owned().await?;
      save(&app, path).await
    }
  });

  let mut models = Vec::with_capacity(set.len());
  while let Some(result) = set.join_next().await {
    if let Ok(Some(model)) = result? {
      models.push(model);
    }
  }

  if !models.is_empty() {
    schedule_cover_extraction(app, models);
  }

  Ok(())
}

pub async fn get_all(app: &AppHandle) -> Result<Vec<LibraryBook>> {
  let mut set = Book::get_all(app)
    .await?
    .into_iter()
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
  I: IntoIterator<Item = book::Model>,
{
  let permits = MAX_FILE_PERMITS / 5;
  let semaphore = Arc::new(Semaphore::new(permits));
  for model in models {
    let app = app.clone();
    let semaphore = Arc::clone(&semaphore);
    spawn(async move {
      let _permit = semaphore.acquire_owned().await?;
      ActiveBook::from_model(&app, &model)?
        .extract_cover(&app)
        .await
    });
  }
}

pub async fn remove(app: &AppHandle, id: i32) -> Result<()> {
  Book::remove(app, id).await?;
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

#[cfg(any(debug_assertions, feature = "devtools"))]
pub async fn remove_all(app: &AppHandle) -> Result<()> {
  Book::remove_all(app).await?;
  Folder::remove_all(app).await?;
  Event::LibraryCleared.emit(app)?;

  let path = app.path().cover_dir()?;
  if let Ok(true) = fs::try_exists(&path).await {
    fs::remove_dir_all(path).await?;
  }

  Ok(())
}

pub async fn scan_book_folders(app: &AppHandle) -> Result<()> {
  let mut books = Vec::new();
  for folder in Folder::get_all(app).await? {
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

/// Adds mock books to the library.
/// This should only be used for testing purposes.
#[cfg(any(debug_assertions, feature = "devtools"))]
pub async fn add_mock_books(
  app: &AppHandle,
  amount: u8,
  size: usize,
  orientation: Orientation,
) -> Result<()> {
  use tokio::task::JoinSet;

  use crate::image::create_mock_book;

  let mut set = JoinSet::new();
  for _ in 0..amount {
    let app = app.clone();
    set.spawn(async move { create_mock_book(&app, size, orientation).await });
  }

  let mut books = Vec::with_capacity(amount.into());
  while let Some(book) = set.join_next().await {
    books.push(book??);
  }

  if !books.is_empty() {
    let path = app.path().mocks_dir()?;
    Folder::create(app, path).await?;
    save_many(app, books).await?;
  }

  Ok(())
}
