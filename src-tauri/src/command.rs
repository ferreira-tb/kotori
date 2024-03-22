use crate::prelude::*;
use tauri_plugin_dialog::{DialogExt, FileDialogBuilder};

#[tauri::command]
pub async fn version(app: AppHandle) -> String {
  app.config().version.clone().unwrap()
}

#[tauri::command]
pub async fn open_file(app: AppHandle, state: State<'_>) -> Result<Option<Json>> {
  let dialog = app.dialog().clone();
  let response = FileDialogBuilder::new(dialog)
    .add_filter("Book", &["cbr", "cbz"])
    .blocking_pick_file();

  if let Some(response) = response {
    match Book::new(&response.path, &app, &state).await {
      Ok(mut book) => {
        book.extract().await?;

        let json = book.as_json()?;
        state.books.lock().await.push(book);

        return Ok(Some(json));
      }
      Err(Error::AlreadyExists) => {
        let mut books = state.books.lock().await;
        let book = books
          .iter_mut()
          .find(|b| b.path == response.path)
          .expect("book should exist");

        book.extract().await?;

        let json = book.as_json()?;
        return Ok(Some(json));
      }
      Err(e) => return Err(e),
    };
  }

  Ok(None)
}
