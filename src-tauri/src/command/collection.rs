use crate::database::entities;
use crate::database::prelude::Collection;
use crate::prelude::*;

#[tauri::command]
pub async fn get_collections(app: AppHandle) -> Result<Vec<entities::collection::Model>> {
  debug!(command = "get_collections");
  Collection::get_all(&app).await
}
