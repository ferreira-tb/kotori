use crate::book::{IntoValue, LibraryBook};
use crate::database::prelude::*;
use crate::prelude::*;
use futures::future::try_join_all;

#[tauri::command]
pub async fn get_library_books(app: AppHandle) -> Result<Value> {
  let kotori = app.state::<Kotori>();
  let books = Book::find().all(&kotori.db).await?;

  let futures = books
    .iter()
    .map(|model| LibraryBook(&app, model).into_value());

  try_join_all(futures).await.map(Value::Array)
}
