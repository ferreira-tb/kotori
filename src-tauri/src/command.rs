use crate::prelude::*;

#[tauri::command]
pub async fn version(app: AppHandle) -> String {
  app.config().package.version.clone().unwrap()
}
