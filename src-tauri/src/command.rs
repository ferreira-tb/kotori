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
    let book = Book::new(path, &app.config());
    if let Ok(book) = book {
      println!("opened book: {:?}", book);
      state.books.lock().unwrap().push(book);
    }
  }

  Ok(())
}
