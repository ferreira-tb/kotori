use crate::library::Library;
use sea_orm::DatabaseConnection;
use std::path::PathBuf;
use std::sync::OnceLock;
use tokio::sync::Mutex;

pub static BOOK_CACHE: OnceLock<PathBuf> = OnceLock::new();

pub struct Database {
  pub conn: DatabaseConnection,
}

pub struct Kotori {
  pub library: Mutex<Library>,
}
