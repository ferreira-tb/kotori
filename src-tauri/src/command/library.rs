use crate::book::{IntoValue, LibraryBook};
use crate::database::prelude::*;
use crate::prelude::*;

#[tauri::command]
pub async fn add_to_library_from_dialog(app: AppHandle) -> Result<()> {
  let kotori = app.state::<Kotori>();
  kotori.library.add_from_dialog().await
}

#[tauri::command]
pub async fn get_library_books(app: AppHandle) -> Result<Value> {
  let kotori = app.state::<Kotori>();
  let books = Book::find().all(&kotori.db).await?;

  let tasks = books.into_iter().map(|model| {
    let app = app.clone();
    async_runtime::spawn(async move {
      let book = LibraryBook(&app, &model).into_value().await;
      book.ok()
    })
  });

  let books = join_all(tasks)
    .await
    .into_iter()
    .filter_map(std::result::Result::unwrap_or_default)
    .collect_vec();

  Ok(Value::Array(books))
}
