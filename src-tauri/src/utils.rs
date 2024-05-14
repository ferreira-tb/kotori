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
  use std::error::Error;
  use tauri::{async_runtime, AppHandle};
  use tauri_plugin_manatsu::Log;

  pub trait ResultExt<T, E: Error> {
    /// Saves an error log, consuming `self`, and discarding the success value, if any.
    fn into_log(self, app: &AppHandle);

    /// Shows an error dialog, consuming `self`, and discarding the success value, if any.
    fn into_dialog(self, app: &AppHandle);
  }

  impl<T, E: Error> ResultExt<T, E> for Result<T, E> {
    fn into_log(self, app: &AppHandle) {
      if let Err(error) = self {
        tracing::error!(%error);

        let app = app.clone();
        let message = error.to_string();
        async_runtime::spawn(async move {
          let _ = Log::new("Error", message)
            .save(&app)
            .await
            .inspect_err(|error| tracing::error!(%error));
        });
      }
    }

    fn into_dialog(self, app: &AppHandle) {
      if let Err(error) = &self {
        dialog::show_error(app, error);
        self.into_log(app);
      }
    }
  }
}
