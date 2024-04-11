mod active;
mod cover;
mod handle;
mod json;
mod title;

pub use active::ActiveBook;
pub use cover::Cover;
pub use json::{IntoJson, LibraryBook, ReaderBook};
pub use title::Title;

use crate::database::prelude::*;
use crate::event::Event;
use crate::prelude::*;
use tauri_plugin_dialog::{DialogExt, FileDialogBuilder};

pub async fn open_from_dialog(app: &AppHandle) -> Result<()> {
  let books = show_dialog(app).await?;

  if !books.is_empty() {
    let kotori = app.state::<Kotori>();
    let reader = kotori.reader.read().await;
    return reader.open_many(books).await.map_err(Into::into);
  }

  Ok(())
}

async fn show_dialog(app: &AppHandle) -> Result<Vec<ActiveBook>> {
  let (tx, rx) = oneshot::channel();
  let dialog = app.dialog().clone();

  FileDialogBuilder::new(dialog)
    .add_filter("Book", &["cbr", "cbz"])
    .pick_files(move |response| {
      tx.send(response).ok();
    });

  if let Some(response) = rx.await? {
    return response
      .into_iter()
      .map(|it| ActiveBook::new(it.path))
      .collect();
  }

  Ok(Vec::new())
}

pub async fn update_rating(app: &AppHandle, id: i32, rating: u8) -> Result<()> {
  if rating > 5 {
    bail!(InvalidRating);
  }

  let kotori = app.state::<Kotori>();
  let book = Book::find_by_id(id)
    .one(&kotori.db)
    .await?
    .ok_or_else(|| err!(BookNotFound))?;

  let mut book: BookActiveModel = book.into();
  book.rating = Set(i32::from(rating));
  book.update(&kotori.db).await?;

  let event = Event::RatingUpdated { id, rating };
  event.emit(app)
}
