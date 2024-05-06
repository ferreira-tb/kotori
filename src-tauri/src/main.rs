// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod book;
mod command;
mod database;
mod error;
mod event;
mod library;
mod macros;
mod menu;
mod prelude;
mod reader;
mod server;
mod utils;

use error::BoxResult;
use reader::Reader;
use sea_orm::DatabaseConnection;
use std::sync::OnceLock;
use tauri::async_runtime::RwLock;
use tauri::{App, AppHandle, Manager, WindowEvent};
use tracing::info;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling;
use tracing_subscriber::fmt::time::ChronoLocal;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};
use utils::app::AppHandleExt;
use utils::date::TIMESTAMP;

const VERSION: &str = env!("CARGO_PKG_VERSION");

static TRACING_GUARD: OnceLock<WorkerGuard> = OnceLock::new();

pub struct Kotori {
  pub db: DatabaseConnection,
  pub reader: RwLock<Reader>,
}

fn main() {
  tauri::Builder::default()
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_http::init())
    .plugin(tauri_plugin_manatsu::init())
    .plugin(tauri_plugin_persisted_scope::init())
    .plugin(tauri_plugin_shell::init())
    .setup(setup)
    .invoke_handler(tauri::generate_handler![
      command::close_window,
      command::focus_main_window,
      command::show_window,
      command::toggle_fullscreen,
      command::library::add_to_library_from_dialog,
      command::library::get_library_books,
      command::library::remove_book,
      command::library::request_remove_book,
      command::library::show_library_book_context_menu,
      command::library::update_book_rating,
      command::reader::delete_book_page,
      command::reader::get_current_reader_book,
      command::reader::get_current_reader_window_id,
      command::reader::open_book,
      command::reader::open_book_from_dialog,
      command::reader::request_delete_page,
      command::reader::show_reader_page_context_menu,
      command::reader::switch_reader_focus,
    ])
    .run(tauri::generate_context!())
    .expect("could not start kotori");
}

fn setup(app: &mut App) -> BoxResult<()> {
  let handle = app.handle();
  setup_tracing(handle)?;

  let kotori = Kotori {
    db: database::connect(handle)?,
    reader: RwLock::new(Reader::new(handle)),
  };

  app.manage(kotori);

  let main_window = handle.get_main_window();
  main_window.set_menu(menu::main::build(handle)?)?;
  main_window.on_menu_event(menu::main::on_event(handle));
  main_window.on_window_event(on_main_window_event(handle));

  // This depends on state managed by Tauri, so it MUST be called after `app.manage`.
  server::serve(handle);

  Ok(())
}

fn setup_tracing(app: &AppHandle) -> BoxResult<()> {
  let filter = EnvFilter::builder()
    .from_env()?
    .add_directive("kotori=trace".parse()?);

  let path = app.path().app_log_dir()?;
  let appender = rolling::never(path, "kotori.log");
  let (file_writer, guard) = tracing_appender::non_blocking(appender);
  TRACING_GUARD.set(guard).unwrap();

  let file_layer = Layer::default()
    .with_ansi(false)
    .with_timer(ChronoLocal::new(TIMESTAMP.into()))
    .with_writer(file_writer.with_max_level(tracing::Level::WARN));

  let stderr_layer = Layer::default()
    .with_ansi(true)
    .with_timer(ChronoLocal::new(TIMESTAMP.into()))
    .with_writer(std::io::stderr)
    .pretty();

  let subscriber = Registry::default()
    .with(file_layer)
    .with(stderr_layer)
    .with(filter);

  tracing::subscriber::set_global_default(subscriber)?;

  Ok(())
}

fn on_main_window_event(app: &AppHandle) -> impl Fn(&WindowEvent) {
  let app = app.clone();
  move |event| {
    if matches!(event, WindowEvent::Destroyed) {
      info!("main window destroyed, exiting");
      app.cleanup_before_exit();
      app.exit(0);
    }
  }
}
