mod app;
mod reader;

pub use app::on_main_window_event;
pub use reader::ReaderWindow;

use crate::prelude::*;
use tauri::{EventTarget, WebviewUrl};

#[derive(Debug)]
pub enum WindowKind {
  Main,
  Reader(u16),
}

impl WindowKind {
  pub fn label(&self) -> String {
    match self {
      Self::Main => "main".into(),
      Self::Reader(id) => format!("reader-{id}"),
    }
  }

  fn data_dir(&self, app: &AppHandle) -> Result<PathBuf> {
    let label = self.label();
    app
      .path()
      .app_local_data_dir()
      .map(|it| it.join(format!("windows/{label}")))
      .map_err(Into::into)
  }

  fn url(&self) -> WebviewUrl {
    let name = match self {
      Self::Main => "main",
      Self::Reader { .. } => "reader",
    };

    WebviewUrl::App(format!("src/windows/{name}/index.html",).into())
  }
}

impl From<WindowKind> for EventTarget {
  fn from(kind: WindowKind) -> Self {
    let label = kind.label();
    EventTarget::WebviewWindow { label }
  }
}
