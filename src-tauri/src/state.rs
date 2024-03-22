use crate::book::Book;
use sea_orm::DatabaseConnection;
use tokio::sync::Mutex;

pub struct Kotori {
  pub books: Mutex<Vec<Book>>,
  pub database: DatabaseConnection,
}

pub type State<'a> = tauri::State<'a, Kotori>;
