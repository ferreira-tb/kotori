use crate::{book, library, prelude::*};

#[tauri::command]
pub async fn add_to_library_from_dialog(app: AppHandle) -> Result<()> {
  info!(name = "add_to_library_from_dialog");
  library::add_from_dialog(&app)
    .await
    .inspect_err(|err| error!("\"{err}\""))
}

#[tauri::command]
pub async fn get_library_books(app: AppHandle) -> Result<Json> {
  info!(name = "get_library_books");
  library::get_all(&app)
    .await
    .inspect_err(|err| error!("\"{err}\""))
}

#[tauri::command]
pub async fn show_library_book_context_menu(app: AppHandle, window: Window, id: i32) -> Result<()> {
  use crate::menu::context::library::book;

  info!(name = "show_library_book_context_menu", book_id = id);

  let menu = book::build(&app)?;
  window.on_menu_event(book::on_event(&app, id));
  menu.popup(window).map_err(Into::into)
}

#[tauri::command]
pub async fn update_book_rating(app: AppHandle, id: i32, rating: u8) -> Result<()> {
  info!(name = "update_book_rating", book_id = id, rating);
  book::update_rating(&app, id, rating)
    .await
    .inspect_err(|err| error!("\"{err}\""))
}
