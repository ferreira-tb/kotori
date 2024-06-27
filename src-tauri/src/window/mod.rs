pub mod app;
mod reader;

pub use reader::ReaderWindow;

use crate::prelude::*;
use strum::{Display, EnumIs};
use tauri::{EventTarget, WebviewUrl};

#[derive(Debug, Display, EnumIs)]
#[strum(serialize_all = "kebab-case")]
pub enum WindowKind {
  Main,
  #[strum(to_string = "reader-{0}")]
  Reader(u16),
}

impl WindowKind {
  pub fn label(&self) -> String {
    self.to_string()
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
