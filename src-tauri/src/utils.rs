pub mod app {
  use crate::book::BookHandle;
  use crate::reader::WindowMap;
  use crate::Kotori;
  use tauri::{AppHandle, Manager, State, WebviewWindow, Wry};

  pub trait AppHandleExt: Manager<Wry> {
    fn kotori(&self) -> State<Kotori> {
      self.state::<Kotori>()
    }

    fn book_handle(&self) -> BookHandle {
      self.kotori().book_handle.clone()
    }

    fn get_focused_window(&self) -> Option<WebviewWindow> {
      self
        .webview_windows()
        .into_values()
        .find(|it| it.is_focused().unwrap_or(false))
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

#[cfg(any(debug_assertions, feature = "devtools"))]
pub mod log {
  use std::io;
  use tauri::{AppHandle, Manager};
  use tracing_appender::non_blocking::WorkerGuard;
  use tracing_appender::rolling;
  use tracing_subscriber::fmt::time::ChronoLocal;
  use tracing_subscriber::fmt::writer::MakeWriterExt;
  use tracing_subscriber::fmt::Layer;
  use tracing_subscriber::layer::SubscriberExt;
  use tracing_subscriber::{EnvFilter, Registry};

  const TIMESTAMP: &str = "%F %T%.3f %:z";

  struct TracingGuard {
    #[allow(dead_code)]
    guard: WorkerGuard,
  }

  pub fn setup_tracing(app: &AppHandle) {
    let filter = EnvFilter::builder()
      .from_env()
      .unwrap()
      .add_directive("kotori=trace".parse().unwrap())
      .add_directive("tauri_plugin_manatsu=trace".parse().unwrap());

    let appender = rolling::never("../", "kotori.log");
    let (writer, guard) = tracing_appender::non_blocking(appender);
    app.manage(TracingGuard { guard });

    let file = Layer::default()
      .with_ansi(false)
      .with_timer(ChronoLocal::new(TIMESTAMP.into()))
      .with_writer(writer.with_max_level(tracing::Level::TRACE))
      .pretty();

    let stderr = Layer::default()
      .with_ansi(true)
      .with_timer(ChronoLocal::new(TIMESTAMP.into()))
      .with_writer(io::stderr)
      .pretty();

    let subscriber = Registry::default()
      .with(file)
      .with(stderr)
      .with(filter);

    tracing::subscriber::set_global_default(subscriber).unwrap();
  }
}

pub mod path {
  use crate::err;
  use crate::error::Result;
  use std::path::{Path, PathBuf};
  use tauri::path::PathResolver;
  use tauri::Wry;

  pub trait PathResolverExt {
    fn cover_dir(&self) -> Result<PathBuf>;
    fn cover(&self, book_id: i32) -> Result<PathBuf>;

    #[cfg(any(debug_assertions, feature = "devtools"))]
    fn dev_cache_dir(&self) -> Result<PathBuf>;
  }

  impl PathResolverExt for PathResolver<Wry> {
    fn cover_dir(&self) -> Result<PathBuf> {
      self
        .app_cache_dir()
        .map(|it| it.join("covers"))
        .map_err(Into::into)
    }

    fn cover(&self, book_id: i32) -> Result<PathBuf> {
      self
        .cover_dir()
        .map(|it| it.join(book_id.to_string()))
    }

    #[cfg(any(debug_assertions, feature = "devtools"))]
    fn dev_cache_dir(&self) -> Result<PathBuf> {
      self
        .app_cache_dir()
        .map(|it| it.join("kotori-dev"))
        .map_err(Into::into)
    }
  }

  pub trait PathExt {
    fn try_parent(&self) -> Result<&Path>;
    fn try_str(&self) -> Result<&str>;
    fn try_string(&self) -> Result<String>;
  }

  impl<P: AsRef<Path>> PathExt for P {
    fn try_parent(&self) -> Result<&Path> {
      let path = self.as_ref();
      path
        .parent()
        .ok_or_else(|| err!(InvalidPath, "{}", path.display()))
    }

    fn try_str(&self) -> Result<&str> {
      let path = self.as_ref();
      path
        .to_str()
        .ok_or_else(|| err!(InvalidPath, "{}", path.display()))
    }

    fn try_string(&self) -> Result<String> {
      self.try_str().map(ToOwned::to_owned)
    }
  }
}

pub mod result {
  use super::dialog;
  use std::error::Error;
  use tauri::{async_runtime, AppHandle};
  use tauri_plugin_manatsu::Log;
  use tracing::error;

  pub trait ResultExt<T, E: Error> {
    /// Saves an error log, consuming `self`, and discarding the success value, if any.
    fn into_log(self, app: &AppHandle);

    /// Shows an error dialog, consuming `self`, and discarding the success value, if any.
    fn into_dialog(self, app: &AppHandle);
  }

  impl<T, E: Error> ResultExt<T, E> for Result<T, E> {
    fn into_log(self, app: &AppHandle) {
      if let Err(err) = self {
        let app = app.clone();
        let message = err.to_string();
        async_runtime::spawn(async move {
          let _ = Log::new("Error", message)
            .save(&app)
            .await
            .inspect_err(|error| error!(%error));
        });
      }
    }

    fn into_dialog(self, app: &AppHandle) {
      if let Err(err) = &self {
        dialog::show_error(app, err);
        self.into_log(app);
      }
    }
  }
}
