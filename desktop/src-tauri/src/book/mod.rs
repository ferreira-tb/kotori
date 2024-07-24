mod active;
mod cover;
mod handle;
mod metadata;
mod structs;
mod title;

use crate::database::{BookExt, BookModelExt};
use crate::event::Event;
use crate::prelude::*;
use crate::reader;
pub use active::ActiveBook;
pub use handle::{BookHandle, MAX_FILE_PERMITS};
use kotori_entity::prelude::Book;
pub use metadata::Metadata;
pub use structs::{LibraryBook, ReaderBook};
use tauri_plugin_dialog::{DialogExt, FileDialogBuilder};
pub use title::Title;
use tokio::sync::oneshot;

pub async fn open_with_dialog(app: &AppHandle) -> Result<()> {
  let (tx, rx) = oneshot::channel();
  let dialog = app.dialog().clone();

  FileDialogBuilder::new(dialog)
    .add_filter("Book", &["cbr", "cbz", "zip"])
    .pick_files(move |response| {
      let _ = tx.send(response);
    });

  let books = rx
    .await?
    .unwrap_or_default()
    .into_iter()
    .filter_map(|it| ActiveBook::new(app, it.path).ok())
    .collect_vec();

  if !books.is_empty() {
    return reader::open_many(app, books)
      .await
      .map_err(Into::into);
  }

  Ok(())
}

/// Set the specified page as the book cover, extracting it afterwards.
pub async fn update_cover<N>(app: &AppHandle, id: i32, name: N) -> Result<()>
where
  N: AsRef<str>,
{
  let model = Book::update_cover(app, id, name).await?;
  let book = ActiveBook::from_model(app, &model)?;
  book.extract_cover(app).await?;
  model.save_as_metadata(app).await
}

pub async fn update_rating(app: &AppHandle, id: i32, rating: u8) -> Result<()> {
  let model = Book::update_rating(app, id, rating).await?;
  Event::RatingUpdated { id, rating }.emit(app)?;
  model.save_as_metadata(app).await
}
