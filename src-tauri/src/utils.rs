pub mod app {
  use tauri::{AppHandle, Manager, State, Wry};

  use crate::book::BookHandle;
  use crate::Kotori;

  pub trait AppHandleExt: Manager<Wry> {
    fn kotori(&self) -> State<Kotori> {
      self.state::<Kotori>()
    }

    fn book_handle(&self) -> BookHandle {
      self.kotori().book_handle.clone()
    }
  }

  impl AppHandleExt for AppHandle {}
}

pub mod collections {
  use std::hash::BuildHasherDefault;

  use ahash::AHasher;
  use indexmap::IndexMap;

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
  use std::path::{Path, PathBuf};

  use tauri::path::PathResolver;
  use tauri::Wry;

  use crate::err;
  use crate::error::Result;

  pub trait PathResolverExt {
    fn cover_dir(&self) -> Result<PathBuf>;
    fn cover(&self, book_id: i32) -> Result<PathBuf>;

    #[cfg(any(debug_assertions, feature = "devtools"))]
    fn dev_cache_dir(&self) -> Result<PathBuf>;
    #[cfg(any(debug_assertions, feature = "devtools"))]
    fn mocks_dir(&self) -> Result<PathBuf>;
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
        .map(|it| it.join("dev-cache"))
        .map_err(Into::into)
    }

    #[cfg(any(debug_assertions, feature = "devtools"))]
    fn mocks_dir(&self) -> Result<PathBuf> {
      self.dev_cache_dir().map(|it| it.join("mocks"))
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
  use std::error::Error;

  use tauri::AppHandle;
  use tauri_plugin_manatsu::Log;
  use tracing::error;

  use crate::utils::dialog;

  pub trait ResultExt<T, E: Error> {
    /// Saves an error log, consuming `self`, and discarding the success value, if any.
    fn log(self, app: &AppHandle);

    /// Shows an error dialog, consuming `self`, and discarding the success value, if any.
    fn dialog(self, app: &AppHandle);
  }

  impl<T, E: Error> ResultExt<T, E> for Result<T, E> {
    fn log(self, app: &AppHandle) {
      if let Err(err) = self {
        let message = err.to_string();
        let _ = Log::new("Error", message)
          .save(app)
          .inspect_err(|error| error!(%error));
      }
    }

    fn dialog(self, app: &AppHandle) {
      if let Err(err) = &self {
        dialog::show_error(app, err);
        self.log(app);
      }
    }
  }
}

pub mod store {
  use std::path::PathBuf;

  use strum::{AsRefStr, Display, EnumString};
  use tauri::{AppHandle, Manager, Wry};
  use tauri_plugin_store::{with_store, Store, StoreCollection};

  use crate::error::Result;

  type StoreResult<T> = std::result::Result<T, tauri_plugin_store::Error>;

  pub trait TauriStore: Manager<Wry> {
    fn with_store<F, T>(&self, path: impl AsRef<str>, f: F) -> Result<T>
    where
      F: FnOnce(&mut Store<Wry>) -> StoreResult<T>,
    {
      let app = self.app_handle().clone();
      let path = PathBuf::from(path.as_ref());
      let collection = self.state::<StoreCollection<Wry>>();
      with_store(app, collection, path, f).map_err(Into::into)
    }

    fn with_config_store<F, T>(&self, f: F) -> Result<T>
    where
      F: FnOnce(&mut Store<Wry>) -> StoreResult<T>,
    {
      self.with_store("config.json", f)
    }
  }

  impl TauriStore for AppHandle {}

  #[derive(AsRefStr, Debug, Display, EnumString)]
  #[strum(serialize_all = "kebab-case")]
  pub enum ConfigKey {
    ColorMode,
  }

  impl From<ConfigKey> for String {
    fn from(value: ConfigKey) -> Self {
      value.to_string()
    }
  }
}

pub mod temp {
  use std::fs::{remove_file, File};
  use std::path::{Path, PathBuf};

  use tracing::trace;
  use uuid::Uuid;

  use crate::error::Result;

  pub struct Tempfile {
    pub path: PathBuf,
    pub file: File,
  }

  impl Tempfile {
    // Create a new temporary file in the specified directory.
    pub fn new_in(dir: impl AsRef<Path>) -> Result<Self> {
      let path = dir.as_ref().join(filename());
      let file = File::create(&path)?;
      Ok(Self { path, file })
    }
  }

  impl Drop for Tempfile {
    fn drop(&mut self) {
      if let Ok(true) = self.path.try_exists() {
        let _ = remove_file(&self.path);
      }

      trace!(tempfile_drop = %self.path.display());
    }
  }

  fn filename() -> String {
    format!("{}.kotori", Uuid::now_v7())
  }
}
