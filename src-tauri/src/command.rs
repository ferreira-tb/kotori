use crate::prelude::*;

#[tauri::command]
pub async fn get_active_book(app: AppHandle, id: u16) -> Result<Value> {
  let kotori = app.state::<Kotori>();
  let reader = kotori.reader.lock().await;
  
  reader
    .get_book_as_value(id)
    .await
    .ok_or_else(|| err!(BookNotFound))
}
