use kotori_entity::collection;
use kotori_entity::prelude::Collection;

use crate::database::CollectionExt;
use crate::prelude::*;

#[tauri::command]
pub async fn get_collections(app: AppHandle) -> Result<Vec<collection::Model>> {
  debug!(command = "get_collections");
  Collection::get_all(&app).await
}
