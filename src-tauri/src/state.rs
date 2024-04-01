use crate::library::Library;
use crate::reader::Reader;
use sea_orm::DatabaseConnection;
use std::path::PathBuf;
use std::sync::OnceLock;
use tauri::async_runtime::Mutex;

pub static BOOK_CACHE: OnceLock<PathBuf> = OnceLock::new();

pub struct Kotori {
  pub db: DatabaseConnection,
  pub library: Mutex<Library>,
  pub reader: Reader,
}
