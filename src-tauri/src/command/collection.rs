use kotori_entity::{collection, prelude::Collection};

use crate::{database::CollectionExt, prelude::*};

#[tauri::command]
pub async fn get_collections(app: AppHandle) -> Result<Vec<collection::Model>> {
  debug!(command = "get_collections");
  Collection::get_all(&app).await
}
