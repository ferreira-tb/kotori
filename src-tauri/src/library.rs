use crate::book::{ActiveBook, LibraryBook, MAX_FILE_PERMITS};
use crate::database::{BookExt, FolderExt};
use crate::event::Event;
use crate::prelude::*;
use crate::utils::glob;
use kotori_entity::book;
use kotori_entity::prelude::{Book, Folder};
use sea_orm::EntityTrait;
use std::sync::Arc;
use tauri_plugin_dialog::{DialogExt, FileDialogBuilder, MessageDialogBuilder, MessageDialogKind};
use tokio::fs;
use tokio::sync::{oneshot, Semaphore};
use tokio::task::JoinSet;
use walkdir::WalkDir;

#[cfg(any(debug_assertions, feature = "devtools"))]
use crate::image::Orientation;

pub async fn add_folders<F>(app: &AppHandle, folders: &[F]) -> Result<()>
where
  F: AsRef<Path>,
{
  if folders.is_empty() {
    return Ok(());
  }

  let globset = glob::book();
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

  if !current_folders.is_empty() {
    let semaphore = Arc::new(Semaphore::new(20));
    let mut set = current_folders
      .into_iter()
      .into_join_set_by(|folder| {
        let app = app.clone();
        let semaphore = Arc::clone(&semaphore);
        async move {
          let _permit = semaphore.acquire_owned().await?;
          Folder::create(&app, folder).await
        }
      });

    while set.join_next().await.is_some() {}
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
  let mut builder = Book::builder(&path);

  // In theory, we should call `BookHandle::close` afterwards, as the file will no longer be needed.
  // But in practice, Kotori will schedule the extraction of the book cover right after it is saved,
  // which would open the file again.
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
    LibraryBook::from_model(app, model)
      .await
      .and_then(|it| Event::BookAdded(&it).emit(app))?;
  }

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

  schedule_cover_extraction(app, models);

  Ok(())
}

pub async fn get_all(app: &AppHandle) -> Result<Vec<LibraryBook>> {
  let semaphore = Arc::new(Semaphore::new(MAX_FILE_PERMITS));
  let mut set = Book::get_all(app)
    .await?
    .into_iter()
    .into_join_set_by(|model| {
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

  schedule_cover_extraction(app, pending);

  Ok(books)
}

fn schedule_cover_extraction(app: &AppHandle, models: Vec<book::Model>) {
  let semaphore = Arc::new(Semaphore::new(MAX_FILE_PERMITS));
  for model in models {
    let app = app.clone();
    let semaphore = Arc::clone(&semaphore);
    spawn(async move {
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
  Book::remove_all(app).await?;
  Event::LibraryCleared.emit(app)?;

  let path = app.path().cover_dir()?;
  fs::remove_dir_all(path).await.map_err(Into::into)
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
    save_many(app, books).await?;
  }

  Ok(())
}
