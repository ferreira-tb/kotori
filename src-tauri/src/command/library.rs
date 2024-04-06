use crate::book::{IntoValue, LibraryBook};
use crate::database::prelude::*;
use crate::prelude::*;
use futures::future::try_join_all;

#[tauri::command]
pub async fn get_library_books(app: AppHandle) -> Result<Value> {
  let start = std::time::Instant::now();
  let kotori = app.state::<Kotori>();
  let books = Book::find().all(&kotori.db).await?;
  let futures = books
    .iter()
    .map(|model| LibraryBook(model).into_value())
    .collect_vec();

  let a = try_join_all(futures).await.map(Value::Array);
  println!("get_library_books took {:?}", start.elapsed());

  a
}
