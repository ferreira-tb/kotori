mod active;
mod cover;
mod handle;
mod structs;
mod title;

pub use active::ActiveBook;
pub use cover::Cover;
pub use structs::{LibraryBook, ReaderBook};
pub use title::Title;

use crate::database::prelude::*;
use crate::event::Event;
use crate::{prelude::*, reader};
use tauri_plugin_dialog::FileDialogBuilder;

pub async fn open_from_dialog(app: &AppHandle) -> Result<()> {
  let (tx, rx) = oneshot::channel();
  let dialog = app.dialog().clone();

  FileDialogBuilder::new(dialog)
    .add_filter("Book", &["cbr", "cbz", "zip"])
    .pick_files(move |response| {
      tx.send(response).ok();
    });

  let books = rx
    .await?
    .unwrap_or_default()
    .into_iter()
    .filter_map(|it| ActiveBook::new(it.path).ok())
    .collect_vec();

  if !books.is_empty() {
    return reader::open_many(app, books)
      .await
      .map_err(Into::into);
  }

  Ok(())
}

pub async fn update_rating(app: &AppHandle, id: i32, rating: u8) -> Result<()> {
  Book::update_rating(app, id, rating).await?;
  Event::RatingUpdated { id, rating }.emit(app)
}
