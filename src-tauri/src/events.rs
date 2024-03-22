use crate::library::Library;
use crate::prelude::*;
use tauri::menu::MenuEvent;
use tokio::task;

pub enum Event {
  AddToLibrary,
  BookOpened,
  NavigateToLibrary,
}

impl Event {
  pub fn as_str(&self) -> &str {
    match self {
      Self::AddToLibrary => "add_to_library",
      Self::BookOpened => "book_opened",
      Self::NavigateToLibrary => "navigate_to_library",
    }
  }
}

#[allow(clippy::needless_pass_by_value)]
pub fn menu_event_handler(app: &AppHandle, event: MenuEvent) {
  match event.id.0.as_str() {
    "library" => {
      app
        .emit(Event::NavigateToLibrary.as_str(), ())
        .ok();
    }
    "open_book" => {
      let app = app.clone();
      task::spawn(async move {
        Library::open_book(&app).await.unwrap();
      });
    }
    _ => {}
  }
}
