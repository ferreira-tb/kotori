use crate::book::{ActiveBook, LibraryBook};
use crate::database::model::{Book, NewFolder};
use crate::event::Event;
use crate::prelude::*;
use crate::utils::glob;
use ahash::{HashSet, HashSetExt};
use future_iter::join_set::IntoJoinSetBy;
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
use tokio::fs;
use tokio::sync::{mpsc, oneshot};
use walkdir::WalkDir;

pub async fn add_with_dialog(app: &AppHandle) -> Result<()> {
  let (tx, rx) = oneshot::channel();
  app.dialog().file().pick_folders(move |response| {
    let _ = tx.send(response.unwrap_or_default());
  });

  let folders = rx.await?;
  add_folders(app, folders).await
}

pub async fn add_folders<I>(app: &AppHandle, folders: I) -> Result<()>
where
  I: IntoIterator<Item = PathBuf>,
{
  let folders = folders.into_iter().unique().collect_vec();
  if folders.is_empty() {
    return Ok(());
  }

  let mut books = HashSet::new();
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
      trace!("skipping folder: {folder:?}");
      continue;
    }

    walk_folder(&mut books, &folder);
    current_folders.push(folder);
  }

  if !current_folders.is_empty() {
    let folders = current_folders
      .into_iter()
      .filter_map(|path| NewFolder::try_from(path).ok())
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

async fn save_many<I>(app: &AppHandle, iter: I) -> Result<()>
where
  I: IntoIterator<Item = PathBuf>,
{
  #[cfg(feature = "tracing")]
  let start = Instant::now();

  let mut books = Vec::new();
  let handle = app.database_handle();
  for book in iter {
    if !handle.has_book_path(&book).await? {
      books.push(book);
    }
  }

  if books.is_empty() {
    return Ok(());
  }

  let (tx, mut rx) = mpsc::channel(100);
  for path in books {
    let app = app.clone();
    let tx = tx.clone();
    spawn(async move {
      let book = save(&app, &path).await;
      let _ = tx.send(book).await;
    });
  }

  drop(tx);

  #[cfg(feature = "tracing")]
  let mut amount = 0;
  while let Some(result) = rx.recv().await {
    #[cfg(feature = "tracing")]
    if result.is_ok() {
      amount += 1;
    }

    result.into_err_log(app);
  }

  #[cfg(feature = "tracing")]
  if amount > 0 {
    let elapsed = start.elapsed();
    info!("{amount} books added to library in {elapsed:?}");
  }

  Ok(())
}

pub async fn save(app: &AppHandle, path: &Path) -> Result<Book> {
  let handle = app.book_handle();
  let mut builder = Book::builder(path);
  if let Some(metadata) = handle.get_metadata(path).await? {
    builder = builder.metadata(metadata);
  }

  let book = builder.save(app).await?;

  // We could already call `BookHandle::set_metadata` to write the metadata of the saved book,
  // but that doesn't seem like a good idea, as the data would only be default values.
  let library_book = LibraryBook::from_book(app, &book);
  Event::BookAdded(&library_book).emit(app)?;

  ActiveBook::from_book(app, &book)
    .extract_cover()
    .await?;

  handle.close(path).await?;

  Ok(book)
}

pub async fn get_all(app: &AppHandle) -> Result<Vec<LibraryBook>> {
  let books = app.database_handle().get_all_books().await?;
  let mut set = books.into_join_set_by(|book| {
    let app = app.clone();
    async move {
      // Remove the book if the file is missing.
      if let Ok(false) = fs::try_exists(&book.path).await {
        remove(&app, book.id).await.into_err_log(&app);
        return None;
      }

      Some(book)
    }
  });

  let mut library_books = Vec::with_capacity(set.len());
  while let Some(result) = set.join_next().await {
    if let Some(book) = result? {
      let library_book = LibraryBook::from_book(app, &book);
      if library_book.cover.is_none() {
        ActiveBook::from_book(app, &book).spawn_extract_cover();
      }

      library_books.push(library_book);
    }
  }

  Ok(library_books)
}

pub async fn remove(app: &AppHandle, id: i32) -> Result<()> {
  app.database_handle().remove_book(id).await?;
  Event::BookRemoved(id).emit(app)?;

  let path = app.path().cover(id);
  if let Ok(true) = fs::try_exists(&path).await {
    fs::remove_file(path).await?;
  }

  Ok(())
}

pub async fn remove_with_dialog(app: &AppHandle, id: i32) -> Result<()> {
  let (tx, rx) = oneshot::channel();
  let title = app.database_handle().get_book_title(id).await?;

  app
    .dialog()
    .message(format!("{title} will be removed from the library."))
    .title("Remove book")
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

  let path = app.path().cover_dir();
  if let Ok(true) = fs::try_exists(&path).await {
    fs::remove_dir_all(path).await?;
  }

  Ok(())
}

pub async fn scan_book_folders(app: &AppHandle) -> Result<()> {
  #[cfg(feature = "tracing")]
  let start = Instant::now();

  let mut books = HashSet::new();
  let folders = app.database_handle().get_all_folders().await?;
  for folder in folders {
    walk_folder(&mut books, &folder);
  }

  if !books.is_empty() {
    save_many(app, books).await?;
  }

  #[cfg(feature = "tracing")]
  info!("book folders scanned in {:?}", start.elapsed());

  Ok(())
}

/// Search recursively for books within the folder.
fn walk_folder(books: &mut HashSet<PathBuf>, folder: &Path) {
  let globset = glob::book();
  for entry in WalkDir::new(folder).into_iter().flatten() {
    let path = entry.into_path();
    if path.is_file() && globset.is_match(&path) {
      books.insert(path);
    }
  }
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
    let path = app.path().mocks_dir();
    app
      .database_handle()
      .save_folders([NewFolder::try_from(path)?])
      .await?;

    save_many(app, books).await?;
  }

  Ok(())
}
