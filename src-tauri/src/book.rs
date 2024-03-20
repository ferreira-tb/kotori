use crate::error::Result;
use std::path::PathBuf;
use tauri::api::path::app_cache_dir;

#[derive(Debug)]
pub struct Book {
  path: PathBuf,
}

impl Book {
  pub fn new(path: PathBuf, config: &tauri::Config) -> Result<Book> {
    let book = Book { path };
    let _ = Book::dir(config);

    Ok(book)
  }

  pub fn dir(config: &tauri::Config) -> Option<PathBuf> {
    app_cache_dir(config).map(|dir| dir.join("books"))
  }
}
