mod active;
mod cover;
mod handle;
mod structs;
mod title;

pub use active::ActiveBook;
pub use cover::Cover;
pub use handle::{BookHandle, MAX_FILE_PERMITS};
pub use structs::{LibraryBook, ReaderBook};
pub use title::Title;

use crate::database::prelude::*;
use crate::event::Event;
use crate::{prelude::*, reader};
use tauri_plugin_dialog::{DialogExt, FileDialogBuilder};
use tokio::fs;
use tokio::sync::oneshot;

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
    .filter_map(|it| ActiveBook::new(app, it.path).ok())
    .collect_vec();

  if !books.is_empty() {
    return reader::open_many(app, books)
      .await
      .map_err(Into::into);
  }

  Ok(())
}

pub async fn get_cover(app: &AppHandle, id: i32) -> Result<Cover> {
  let path = app.path().cover(id)?;
  if fs::try_exists(&path).await? {
    return Ok(path.into());
  }

  Ok(Cover::NotExtracted)
}

pub async fn update_rating(app: &AppHandle, id: i32, rating: u8) -> Result<()> {
  Book::update_rating(app, id, rating).await?;
  Event::RatingUpdated { id, rating }.emit(app)
}
