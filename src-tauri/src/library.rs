use crate::book::{ActiveBook, Cover, IntoJson, LibraryBook, Title};
use crate::database::prelude::*;
use crate::event::Event;
use crate::prelude::*;
use crate::utils::{self, glob};
use tauri_plugin_dialog::{DialogExt, FileDialogBuilder, MessageDialogBuilder, MessageDialogKind};
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

  save_books(app, books).await
}

pub async fn get_books(app: &AppHandle) -> Result<Json> {
  let kotori = app.state::<Kotori>();
  let books = Book::find().all(&kotori.db).await?;

  let tasks = books.into_iter().map(|model| {
    let app = app.clone();
    async_runtime::spawn(async move {
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
    })
  });

  let books = join_all(tasks)
    .await
    .into_iter()
    .filter_map(std::result::Result::unwrap_or_default)
    .collect_vec();

  Ok(Json::Array(books))
}

pub async fn remove_book(app: &AppHandle, id: i32) -> Result<()> {
  let kotori = app.state::<Kotori>();
  let book = Book::find_by_id(id)
    .one(&kotori.db)
    .await?
    .ok_or_else(|| err!(BookNotFound))?;

  let title = "Are you sure?";
  let message = format!(
    "{} will be removed from the library.",
    Title::try_from(book.path.as_str())?
  );

  let dialog = app.dialog().clone();
  let dialog = MessageDialogBuilder::new(dialog, title, message)
    .kind(MessageDialogKind::Warning)
    .ok_button_label("Remove")
    .cancel_button_label("Cancel");

  let (tx, rx) = oneshot::channel();
  dialog.show(move |response| {
    tx.send(response).ok();
  });

  if rx.await? {
    Book::delete_by_id(id).exec(&kotori.db).await?;
    return Event::BookRemoved(id).emit(app);
  };

  Ok(())
}

async fn save_book(app: &AppHandle, path: &Path) -> Result<()> {
  let path = utils::path::to_string(path)?;
  let model = BookActiveModel {
    id: NotSet,
    path: Set(path),
    rating: NotSet,
    cover: NotSet,
  };

  let on_conflict = OnConflict::column(BookColumn::Path)
    .do_nothing()
    .to_owned();

  let kotori = app.state::<Kotori>();
  let book = Book::insert(model)
    .on_conflict(on_conflict)
    .exec_with_returning(&kotori.db)
    .await?;

  let payload = LibraryBook(app, &book).into_json().await?;
  Event::BookAdded(payload).emit(app)?;

  let active_book = ActiveBook::with_model(&book)?;
  let cover = Cover::path(app, book.id)?;
  active_book.extract_cover(app, cover);

  Ok(())
}

async fn save_books<I>(app: &AppHandle, paths: I) -> Result<()>
where
  I: IntoIterator<Item = PathBuf>,
{
  let tasks = paths.into_iter().map(|path| {
    let app = app.clone();
    async_runtime::spawn(async move {
      save_book(&app, &path).await?;
      Ok::<(), Error>(())
    })
  });

  join_all(tasks).await;

  Ok(())
}
