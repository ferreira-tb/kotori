use crate::{prelude::*, reader};
use strum::{Display, EnumString};
use tauri::EventTarget;

#[derive(Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum Event {
  BookAdded(Json),
  BookRemoved(i32),
  CoverExtracted { id: i32, path: PathBuf },
  DeletePageRequested { window_id: u16, page: usize },
  PageDeleted { window_id: u16 },
  RatingUpdated { id: i32, rating: u8 },
  RemoveBookRequested { id: i32, title: String },
}

impl Event {
  pub fn emit(self, app: &AppHandle) -> Result<()> {
    let event = self.to_string();

    macro_rules! to_reader {
      ($id:expr) => {{
        self.emit_to_reader(app, &event, $id)
      }};
    }

    match self {
      Event::DeletePageRequested { window_id, .. } => to_reader!(window_id),
      Event::PageDeleted { window_id, .. } => to_reader!(window_id),
      _ => self.emit_to_main(app, &event),
    }
  }

  fn emit_to_main(self, app: &AppHandle, event: &str) -> Result<()> {
    debug!(event, target = "main");
    app
      .emit_to(Target::MainWindow, event, Json::from(self))
      .map_err(Into::into)
  }

  fn emit_to_reader(self, app: &AppHandle, event: &str, window_id: u16) -> Result<()> {
    debug!(event, target = "reader", reader_id = window_id);
    app
      .emit_to(Target::ReaderWindow(window_id), event, Json::from(self))
      .map_err(Into::into)
  }
}

impl From<Event> for Json {
  fn from(event: Event) -> Self {
    match event {
      Event::BookAdded(value) => value,
      Event::BookRemoved(id) => json!({ "id": id }),
      Event::CoverExtracted { id, path } => json!({ "id": id, "path": path }),
      Event::DeletePageRequested { page, .. } => json!({ "page": page }),
      Event::PageDeleted { .. } => Json::Null,
      Event::RatingUpdated { id, rating } => json!({ "id": id, "rating": rating }),
      Event::RemoveBookRequested { id, title } => json!({ "id": id, "title": title }),
    }
  }
}

#[derive(Debug)]
pub enum Target {
  MainWindow,
  ReaderWindow(u16),
}

impl Target {
  pub fn label(&self) -> String {
    match self {
      Target::MainWindow => "main".into(),
      Target::ReaderWindow(window_id) => reader::label(*window_id),
    }
  }
}

impl From<Target> for EventTarget {
  fn from(target: Target) -> Self {
    EventTarget::WebviewWindow {
      label: target.label(),
    }
  }
}
