use crate::book::LibraryBook;
use crate::library;
use crate::prelude::*;

#[tauri::command]
pub async fn add_to_library_with_dialog(app: AppHandle) -> Result<()> {
  #[cfg(feature = "tracing")]
  debug!(command = "add_to_library_with_dialog");

  library::add_with_dialog(&app).await
}

#[tauri::command]
pub async fn get_library_books(app: AppHandle) -> Result<Vec<LibraryBook>> {
  #[cfg(feature = "tracing")]
  debug!(command = "get_library_books");

  library::get_all(&app).await
}

#[tauri::command]
pub async fn remove_book(app: AppHandle, id: i32) -> Result<()> {
  #[cfg(feature = "tracing")]
  debug!(command = "remove_book", id);

  library::remove(&app, id).await
}

#[tauri::command]
pub async fn remove_book_with_dialog(app: AppHandle, id: i32) -> Result<()> {
  #[cfg(feature = "tracing")]
  debug!(command = "remove_book_with_dialog", id);

  library::remove_with_dialog(&app, id).await
}

#[tauri::command]
pub async fn show_library_book_context_menu(window: Window, book_id: i32) -> Result<()> {
  use crate::menu::context::library_book::{Context, LibraryBookContextMenu};

  #[cfg(feature = "tracing")]
  debug!(
    command = "show_library_book_context_menu",
    window = window.label(),
    book_id
  );

  let ctx = Context::new(&window, book_id).await?;
  LibraryBookContextMenu::popup(&window, ctx)
}

#[tauri::command]
pub async fn update_book_rating(app: AppHandle, id: i32, rating: u8) -> Result<()> {
  #[cfg(feature = "tracing")]
  debug!(command = "update_book_rating", book_id = id, rating);

  app
    .database_handle()
    .update_book_rating(id, rating)
    .await
    .map(drop)
}
