use crate::database::prelude::*;
use crate::event::Event;
use crate::{book, library, prelude::*};

#[tauri::command]
pub async fn add_to_library_from_dialog(app: AppHandle) -> Result<()> {
  debug!(name = "add_to_library_from_dialog");
  library::add_from_dialog(&app)
    .await
    .inspect_err(|err| error!("{err}"))
}

#[tauri::command]
pub async fn get_library_books(app: AppHandle) -> Result<Json> {
  debug!(name = "get_library_books");
  library::get_all(&app)
    .await
    .inspect_err(|err| error!("{err}"))
}

#[tauri::command]
pub async fn remove_book(app: AppHandle, id: i32) -> Result<()> {
  debug!(name = "remove_book", book_id = id);
  library::remove(&app, id)
    .await
    .inspect_err(|err| error!("{err}"))
}

#[tauri::command]
pub async fn request_remove_book(app: AppHandle, id: i32) -> Result<()> {
  debug!(name = "request_remove_book", book_id = id);
  let title = Book::get_title(&app, id).await?.to_string();
  Event::RemoveBookRequested { id, title }.emit(&app)
}

#[tauri::command]
pub async fn show_library_book_context_menu(app: AppHandle, window: Window, id: i32) -> Result<()> {
  use crate::menu::context::library::book;

  debug!(name = "show_library_book_context_menu", book_id = id);
  let menu = book::build(&app)?;
  window.on_menu_event(book::on_event(&app, id));
  menu.popup(window).map_err(Into::into)
}

#[tauri::command]
pub async fn update_book_rating(app: AppHandle, id: i32, rating: u8) -> Result<()> {
  debug!(name = "update_book_rating", book_id = id, rating);
  book::update_rating(&app, id, rating)
    .await
    .inspect_err(|err| error!("{err}"))
}
