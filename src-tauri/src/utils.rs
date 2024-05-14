pub mod app {
  use crate::reader::WindowMap;
  use crate::Kotori;
  use tauri::{AppHandle, Manager, Runtime, State, WebviewWindow};

  pub trait AppHandleExt<R: Runtime>: Manager<R> {
    fn kotori(&self) -> State<Kotori> {
      self.state::<Kotori>()
    }

    fn main_window(&self) -> WebviewWindow<R> {
      self.get_webview_window("main").unwrap()
    }

    fn reader_windows(&self) -> WindowMap {
      self.kotori().reader.windows()
    }
  }

  impl<R: Runtime> AppHandleExt<R> for AppHandle<R> {}
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

pub mod path {
  use crate::err;
  use crate::error::Result;
  use std::path::Path;

  pub fn parent(path: &Path) -> Result<&Path> {
    path
      .parent()
      .ok_or_else(|| err!(InvalidPath, "{}", path.display()))
  }

  pub fn to_str(path: &Path) -> Result<&str> {
    path
      .to_str()
      .ok_or_else(|| err!(InvalidPath, "{}", path.display()))
  }

  pub fn to_string(path: impl AsRef<Path>) -> Result<String> {
    to_str(path.as_ref()).map(ToOwned::to_owned)
  }
}

pub mod result {
  use super::dialog;
  use std::fmt::Display;
  use tauri::AppHandle;

  pub trait ResultExt<T, E: Display> {
    /// Show an message dialog if the result is an error.
    ///
    /// This method will also log the error using the [`tracing::error`] macro.
    fn show_dialog_on_error(self, app: &AppHandle) -> Result<T, E>;
  }

  impl<T, E: Display> ResultExt<T, E> for Result<T, E> {
    fn show_dialog_on_error(self, app: &AppHandle) -> Result<T, E> {
      if let Err(error) = &self {
        tracing::error!(%error);
        dialog::show_error(app, error);
      }

      self
    }
  }
}
