use crate::database::model::Collection;
use crate::prelude::*;

#[tauri::command]
pub async fn get_collections(app: AppHandle) -> Result<Vec<Collection>> {
  #[cfg(feature = "tracing")]
  debug!(command = "get_collections");

  app.database_handle().get_all_collections().await
}
