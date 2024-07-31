pub mod collections {
  use ahash::AHasher;
  use indexmap::IndexMap;
  use std::hash::BuildHasherDefault;

  pub type OrderedMap<K, V> = IndexMap<K, V, BuildHasherDefault<AHasher>>;
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

#[cfg(feature = "tracing")]
pub mod log {
  use crate::result::BoxResult;
  use std::io;
  use tauri::{AppHandle, Manager};
  use tracing::subscriber::set_global_default;
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

  pub fn setup_tracing(app: &AppHandle) -> BoxResult<()> {
    let filter = EnvFilter::builder()
      .from_env()?
      .add_directive("kotori=trace".parse()?)
      .add_directive("tauri_plugin_manatsu=trace".parse()?)
      .add_directive("tauri_plugin_pinia=trace".parse()?);

    let appender = rolling::never("../../", "kotori.log");
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

    set_global_default(subscriber).map_err(Into::into)
  }
}

pub mod manager {
  use crate::book::BookHandle;
  use crate::database::DatabaseHandle;
  use crate::Kotori;
  use tauri::{AppHandle, Manager, State, WebviewWindow, Window, Wry};

  pub trait ManagerExt: Manager<Wry> {
    fn kotori(&self) -> State<Kotori> {
      self.state::<Kotori>()
    }

    fn book_handle(&self) -> BookHandle {
      self.kotori().book_handle.clone()
    }

    fn database_handle(&self) -> DatabaseHandle {
      self.kotori().database_handle.clone()
    }
  }

  impl ManagerExt for AppHandle {}
  impl ManagerExt for WebviewWindow {}
  impl ManagerExt for Window {}
}
