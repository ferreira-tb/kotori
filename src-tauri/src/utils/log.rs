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
  _guard: WorkerGuard,
}

pub fn setup_tracing(app: &AppHandle) {
  #[cfg_attr(not(feature = "tokio-console"), allow(unused_mut))]
  let mut filter = EnvFilter::builder()
    .from_env()
    .unwrap()
    .add_directive("kotori=trace".parse().unwrap())
    .add_directive("tauri_plugin_manatsu=trace".parse().unwrap());

  #[cfg(feature = "tokio-console")]
  {
    filter = filter
      .add_directive("tokio=trace".parse().unwrap())
      .add_directive("runtime=trace".parse().unwrap());
  }

  let appender = rolling::never("../", ".log");
  let (writer, guard) = tracing_appender::non_blocking(appender);
  app.manage(TracingGuard { _guard: guard });

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

  macro_rules! set_global_default {
    ($($layer:expr),*) => {{
      let subscriber = Registry::default()$(.with($layer))*.with(filter);
      tracing::subscriber::set_global_default(subscriber).unwrap();
    }};
  }

  #[cfg(feature = "tokio-console")]
  set_global_default!(console_subscriber::spawn(), file, stderr);
  #[cfg(not(feature = "tokio-console"))]
  set_global_default!(file, stderr);
}
