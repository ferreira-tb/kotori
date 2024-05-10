use crate::book::{ActiveBook, Cover, IntoJson, LibraryBook};
use crate::database::prelude::*;
use crate::event::Event;
use crate::prelude::*;
use crate::utils::{self, glob};
use tauri_plugin_dialog::{FileDialogBuilder, MessageDialogBuilder, MessageDialogKind};
use walkdir::WalkDir;

pub async fn add_from_dialog(app: &AppHandle) -> Result<()> {
  let (tx, rx) = oneshot::channel();
  let dialog = app.dialog().clone();

  FileDialogBuilder::new(dialog).pick_folders(move |response| {
    tx.send(response.unwrap_or_default()).ok();
  });

  let folders = rx.await?;
  if folders.is_empty() {
    return Ok(());
  }

  let globset = glob::book();
  let mut books = Vec::new();

  for folder in folders {
    for entry in WalkDir::new(&folder).into_iter().flatten() {
      let path = entry.into_path();
      if path.is_file() && globset.is_match(&path) {
        books.push(path);
      }
    }
  }

  if books.is_empty() {
    return Ok(());
  }

  save_many(app, books).await
}

pub async fn get_all(app: &AppHandle) -> Result<Json> {
  let kotori = app.kotori();
  let books = Book::find()
    .all(&kotori.db)
    .await?
    .into_co_stream()
    .map(|model| to_library_book(app.clone(), model))
    .collect::<Vec<_>>()
    .await
    .into_iter()
    .flatten()
    .collect();

  Ok(Json::Array(books))
}

async fn to_library_book(app: AppHandle, model: BookModel) -> Option<Json> {
  // Remove the book if the file is missing.
  if let Ok(false) = fs::try_exists(&model.path).await {
    remove(&app, model.id).await.ok();
    return None;
  }

  let json = LibraryBook(&app, &model).into_json().await;
  if matches!(json, Ok(ref it) if it.get("cover").is_some_and(Json::is_null)) {
    let Ok(book) = ActiveBook::with_model(&model) else {
      return json.ok();
    };

    if let Ok(cover) = Cover::path(&app, model.id) {
      book.extract_cover(&app, cover);
    }
  }

  json.ok()
}

pub async fn remove(app: &AppHandle, id: i32) -> Result<()> {
  let kotori = app.kotori();
  Book::delete_by_id(id).exec(&kotori.db).await?;
  Event::BookRemoved(id).emit(app)?;

  if let Ok(cover) = Cover::path(app, id) {
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
      tx.send(response).ok();
    });

  if let Ok(true) = rx.await {
    remove(app, id).await?;
  }

  Ok(())
}

async fn save(app: AppHandle, path: impl AsRef<Path>) -> Result<()> {
  let path = utils::path::to_string(path)?;
  let model = BookActiveModel {
    path: Set(path),
    ..Default::default()
  };

  let kotori = app.kotori();
  let book = Book::insert(model)
    .on_conflict(
      OnConflict::column(BookColumn::Path)
        .do_nothing()
        .to_owned(),
    )
    .exec_with_returning(&kotori.db)
    .await?;

  let payload = LibraryBook(&app, &book).into_json().await?;
  Event::BookAdded(payload).emit(&app)?;

  let active_book = ActiveBook::with_model(&book)?;
  let cover = Cover::path(&app, book.id)?;
  active_book.extract_cover(&app, cover);

  Ok(())
}

async fn save_many<I>(app: &AppHandle, books: I) -> Result<()>
where
  I: IntoConcurrentStream<Item = PathBuf>,
{
  books
    .into_co_stream()
    .map(|path| save(app.clone(), path))
    .collect::<Vec<_>>()
    .await;

  Ok(())
}
