use crate::prelude::*;
use crate::{book, library};

#[tauri::command]
pub async fn add_to_library_from_dialog(app: AppHandle) -> Result<()> {
  library::add_from_dialog(&app).await
}

#[tauri::command]
pub async fn get_library_books(app: AppHandle) -> Result<Json> {
  library::get_all(&app).await
}

#[tauri::command]
pub async fn show_library_book_context_menu(app: AppHandle, window: Window, id: i32) -> Result<()> {
  use crate::menu::context::library::book;

  let menu = book::build(&app)?;
  window.on_menu_event(book::on_event(&app, id));
  menu.popup(window)?;

  Ok(())
}

#[tauri::command]
pub async fn update_book_rating(app: AppHandle, id: i32, rating: u8) -> Result<()> {
  book::update_rating(&app, id, rating).await
}
