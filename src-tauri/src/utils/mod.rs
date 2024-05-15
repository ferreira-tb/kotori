#[cfg(any(debug_assertions, feature = "devtools"))]
pub mod log;
pub mod path;
pub mod result;

pub mod app {
  use crate::reader::WindowMap;
  use crate::Kotori;
  use tauri::{AppHandle, Manager, State, WebviewWindow, Wry};

  pub trait AppHandleExt: Manager<Wry> {
    fn kotori(&self) -> State<Kotori> {
      self.state::<Kotori>()
    }

    fn main_window(&self) -> WebviewWindow {
      self.get_webview_window("main").unwrap()
    }

    fn reader_windows(&self) -> WindowMap {
      self.kotori().reader.windows()
    }
  }

  impl AppHandleExt for AppHandle {}
}

pub mod collections {
  use ahash::AHasher;
  use indexmap::IndexMap;
  use std::hash::BuildHasherDefault;

  pub type OrderedMap<K, V> = IndexMap<K, V, BuildHasherDefault<AHasher>>;
}

pub mod dialog {
  use std::fmt::Display;
  use tauri::AppHandle;
  use tauri_plugin_dialog::{DialogExt, MessageDialogBuilder, MessageDialogKind};

  pub fn show_error(app: &AppHandle, error: impl Display) {
    let dialog = app.dialog().clone();
    MessageDialogBuilder::new(dialog, "Error", error.to_string())
      .kind(MessageDialogKind::Error)
      .show(|_| {});
  }
}

pub mod glob {
  use globset::{Glob, GlobBuilder, GlobSet, GlobSetBuilder};

  fn glob(glob: &str) -> Glob {
    GlobBuilder::new(glob)
      .case_insensitive(true)
      .build()
      .unwrap()
  }

  pub fn book() -> GlobSet {
    GlobSetBuilder::new()
      .add(glob("*.cbr"))
      .add(glob("*.cbz"))
      .add(glob("*.zip"))
      .build()
      .unwrap()
  }

  pub fn book_page() -> GlobSet {
    GlobSetBuilder::new()
      .add(glob("*.bmp"))
      .add(glob("*.gif"))
      .add(glob("*.jpg"))
      .add(glob("*.jpeg"))
      .add(glob("*.png"))
      .add(glob("*.webp"))
      .build()
      .unwrap()
  }
}
