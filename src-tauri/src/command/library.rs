use crate::book::LibraryBook;
use crate::library;
use crate::prelude::*;

#[tauri::command]
pub async fn add_to_library_with_dialog(app: AppHandle) -> Result<()> {
  debug!(command = "add_to_library_with_dialog");
  library::add_with_dialog(&app).await
}

#[tauri::command]
pub async fn get_library_books(app: AppHandle) -> Result<Vec<LibraryBook>> {
  debug!(command = "get_library_books");
  library::get_all(&app).await
}

#[tauri::command]
pub async fn remove_book(app: AppHandle, id: i32) -> Result<()> {
  debug!(command = "remove_book", id);
  library::remove(&app, id).await
}

#[tauri::command]
pub async fn remove_book_with_dialog(app: AppHandle, id: i32) -> Result<()> {
  debug!(command = "remove_book_with_dialog", id);
  library::remove_with_dialog(&app, id).await
}

#[tauri::command]
pub async fn show_library_book_context_menu(window: Window, book_id: i32) -> Result<()> {
  use crate::menu::context::library::book;
  use crate::menu::context::library::book::{Context, LibraryBookContextMenu};

  debug!(
    command = "show_library_book_context_menu",
    window = window.label(),
    book_id
  );

  let ctx = Context { book_id };
  if let Some(state) = window.try_state::<LibraryBookContextMenu>() {
    *state.ctx.lock().unwrap() = ctx;
    window.popup_menu(&state.menu)?;
  } else {
    let menu = book::build(&window)?;
    window.popup_menu(&menu)?;

    let ctx = std::sync::Mutex::new(ctx);
    window.manage(LibraryBookContextMenu { menu, ctx });
  }

  Ok(())
}

#[tauri::command]
pub async fn update_book_rating(app: AppHandle, id: i32, rating: u8) -> Result<()> {
  debug!(command = "update_book_rating", book_id = id, rating);
  crate::book::update_rating(&app, id, rating).await
}
