// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(try_blocks)]

mod book;
mod command;
mod database;
mod error;
mod event;
mod library;
mod menu;
mod prelude;
mod reader;
mod server;
mod utils;
mod window;

use error::BoxResult;
use reader::Reader;
use sea_orm::DatabaseConnection;
use tauri::{App, Manager};
use utils::app::AppHandleExt;
use window::app::{on_menu_event, on_window_event};

#[cfg(any(debug_assertions, feature = "devtools"))]
use tracing_appender::non_blocking::WorkerGuard;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Kotori {
  pub db: DatabaseConnection,
  pub reader: Reader,
}

fn main() {
  #[cfg(any(debug_assertions, feature = "devtools"))]
  let _guard = setup_tracing();

  tauri::Builder::default()
    .plugin(tauri_plugin_clipboard_manager::init())
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_http::init())
    .plugin(tauri_plugin_manatsu::init())
    .plugin(tauri_plugin_persisted_scope::init())
    .plugin(tauri_plugin_shell::init())
    .plugin(tauri_plugin_store::Builder::new().build())
    .setup(setup)
    .invoke_handler(tauri::generate_handler![
      command::close_window,
      command::focus_main_window,
      command::maximize_window,
      command::show_window,
      command::toggle_fullscreen,
      command::collection::get_collections,
      command::library::add_to_library_from_dialog,
      command::library::get_library_books,
      command::library::remove_book,
      command::library::remove_book_with_dialog,
      command::library::show_library_book_context_menu,
      command::library::update_book_rating,
      command::reader::delete_page_with_dialog,
      command::reader::get_current_reader_book,
      command::reader::open_book,
      command::reader::open_book_from_dialog,
      command::reader::show_reader_page_context_menu,
      command::reader::switch_reader_focus,
      command::reader::toggle_reader_menu
    ])
    .run(tauri::generate_context!())
    .expect("could not start kotori");
}

fn setup(app: &mut App) -> BoxResult<()> {
  let app = app.handle();
  let kotori = Kotori {
    db: database::connect(app)?,
    reader: Reader::new(),
  };

  app.manage(kotori);

  let main_window = app.main_window();
  main_window.set_menu(menu::app::build(app)?)?;
  main_window.on_menu_event(on_menu_event());
  main_window.on_window_event(on_window_event(app));

  #[cfg(debug_assertions)]
  main_window.open_devtools();

  // This depends on state managed by Tauri, so it MUST be called after `app.manage`.
  server::serve(app);

  Ok(())
}

#[cfg(any(debug_assertions, feature = "devtools"))]
fn setup_tracing() -> WorkerGuard {
  use tracing_appender::rolling;
  use tracing_subscriber::fmt::time::ChronoLocal;
  use tracing_subscriber::fmt::writer::MakeWriterExt;
  use tracing_subscriber::fmt::Layer;
  use tracing_subscriber::layer::SubscriberExt;
  use tracing_subscriber::{EnvFilter, Registry};

  const TIMESTAMP: &str = "%F %T%.3f %:z";

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

  let appender = rolling::daily("../.temp", "kotori.log");
  let (writer, guard) = tracing_appender::non_blocking(appender);

  let file = Layer::default()
    .with_ansi(false)
    .with_timer(ChronoLocal::new(TIMESTAMP.into()))
    .with_writer(writer.with_max_level(tracing::Level::TRACE))
    .pretty();

  let stderr = Layer::default()
    .with_ansi(true)
    .with_timer(ChronoLocal::new(TIMESTAMP.into()))
    .with_writer(std::io::stderr)
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

  guard
}
