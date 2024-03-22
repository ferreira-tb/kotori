use crate::book::Book;
use tauri::menu::MenuEvent;
use tauri::{AppHandle, Manager};
use tokio::task;

pub enum Event {
  BookOpened,
  NavigateToLibrary,
}

impl Event {
  pub fn as_str(&self) -> &str {
    match self {
      Self::BookOpened => "book_opened",
      Self::NavigateToLibrary => "navigate_to_library",
    }
  }
}

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
        Book::open(&app).await.unwrap();
      });
    }
    _ => {}
  }
}
