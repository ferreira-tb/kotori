use crate::library::Library;
use crate::reader::Reader;
use sea_orm::DatabaseConnection;
use tauri::async_runtime::Mutex;

pub struct Kotori {
  pub db: DatabaseConnection,
  pub library: Mutex<Library>,
  pub reader: Mutex<Reader>,
}
