use crate::prelude::*;
use tauri::api::dialog::blocking::FileDialogBuilder;

#[tauri::command]
pub async fn version(app: AppHandle) -> String {
  app.config().package.version.clone().unwrap()
}

#[tauri::command]
pub async fn open_file(app: AppHandle, state: State<'_>) -> Result<()> {
  let path = FileDialogBuilder::new()
    .add_filter("Book", &["cbr", "cbz"])
    .pick_file();

  if let Some(path) = path {
    match Book::new(&path, &app.config(), &state).await {
      Ok(mut book) => {
        book.open().await?;
        state.books.lock().await.push(book);
      }
      Err(Error::AlreadyExists) => {
        let mut books = state.books.lock().await;
        let book = books
          .iter_mut()
          .find(|b| b.path == path)
          .expect("book should exist");

        book.open().await?;
      }
      Err(e) => return Err(e),
    };
  }

  Ok(())
}
