mod active;
mod cover;
mod handle;
mod metadata;
mod structs;
mod title;

use crate::prelude::*;
use crate::reader;
pub use active::ActiveBook;
pub use handle::BookHandle;
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
