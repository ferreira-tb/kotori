use crate::prelude::*;
use strum::{Display, EnumString};
use tauri::EventTarget;

#[derive(Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum Event {
  BookAdded(Json),
  CoverExtracted { id: i32, path: PathBuf },
  RatingUpdated { id: i32, rating: u8 },
}

impl Event {
  pub fn emit(self, app: &AppHandle) -> Result<()> {
    let event = self.to_string();
    let payload = Json::from(self);
    app
      .emit_to(Target::MainWindow, &event, payload)
      .map_err(Into::into)
  }
}

impl From<Event> for Json {
  fn from(event: Event) -> Self {
    match event {
      Event::BookAdded(value) => value,
      Event::CoverExtracted { id, path } => json!({ "id": id, "path": path }),
      Event::RatingUpdated { id, rating } => json!({ "id": id, "rating": rating }),
    }
  }
}

enum Target {
  MainWindow,
}

impl From<Target> for EventTarget {
  fn from(target: Target) -> Self {
    match target {
      Target::MainWindow => EventTarget::WebviewWindow {
        label: "main".into(),
      },
    }
  }
}
