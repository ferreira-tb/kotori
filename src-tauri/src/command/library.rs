use crate::book::{self, LibraryBook};

use crate::{library, prelude::*};

#[tauri::command]
pub async fn add_to_library_from_dialog(app: AppHandle) -> Result<()> {
  debug!(command = "add_to_library_from_dialog");
  library::add_from_dialog(&app).await
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
pub async fn show_library_book_context_menu(app: AppHandle, window: Window, id: i32) -> Result<()> {
  use crate::menu::context::library::book::{self, Context, Item};
  use crate::menu::Listener;

  debug!(
    command = "show_library_book_context_menu",
    window = window.label(),
    book_id = id
  );

  let ctx = Context { book_id: id };
  let menu = book::build(&app)?;
  window.on_menu_event(Item::on_event(app, ctx));
  window.popup_menu(&menu).map_err(Into::into)
}

#[tauri::command]
pub async fn update_book_rating(app: AppHandle, id: i32, rating: u8) -> Result<()> {
  debug!(command = "update_book_rating", book_id = id, rating);
  book::update_rating(&app, id, rating).await
}
