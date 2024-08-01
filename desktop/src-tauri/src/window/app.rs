use super::{ColorMode, WindowKind};
use crate::book::ActiveBook;
use crate::menu::AppMenu;
use crate::reader;
use crate::result::{Result, ResultExt};
use crate::utils::glob;
use itertools::Itertools;
use std::path::PathBuf;
use tauri::async_runtime::spawn;
use tauri::menu::MenuEvent;
use tauri::{AppHandle, DragDropEvent, WebviewWindowBuilder, Window, WindowEvent};

pub fn open(app: &AppHandle) -> Result<()> {
  let kind = WindowKind::Main;
  let window = WebviewWindowBuilder::new(app, kind.label(), kind.url())
    .data_directory(kind.data_dir(app)?)
    .title("Kotori")
    .theme(ColorMode::get(app)?.into())
    .resizable(true)
    .maximizable(true)
    .minimizable(true)
    .visible(false)
    .build()?;

  window.set_menu(AppMenu::build(app)?)?;
  window.on_menu_event(on_menu_event());
  window.on_window_event(on_window_event(app));

  #[cfg(feature = "devtools")]
  window.set_title(&format!("Kotori DEV {}", crate::VERSION))?;

  #[cfg(feature = "open-main-devtools")]
  window.open_devtools();

  AppMenu::spawn_update(app);

  Ok(())
}

/// Calling `on_menu_event` on a window will override previously registered event listeners.
/// For this reason, all listeners must be registered inside a single call.
fn on_menu_event() -> impl Fn(&Window, MenuEvent) {
  use crate::menu::{AppMenuItem, LibraryBookContextMenuItem, Listener};
  move |window, event| {
    AppMenuItem::execute(window, &event);
    LibraryBookContextMenuItem::execute(window, &event);
  }
}

fn on_window_event(app: &AppHandle) -> impl Fn(&WindowEvent) {
  let app = app.clone();
  move |event| match event {
    WindowEvent::Destroyed => {
      #[cfg(feature = "tracing")]
      tracing::info!("main window destroyed, exiting");

      app.exit(0);
    }
    WindowEvent::DragDrop(DragDropEvent::Drop { paths, .. }) => {
      #[cfg(feature = "tracing")]
      tracing::trace!(?paths, "dropped files");

      handle_drop_event(&app, paths);
    }
    _ => {}
  }
}

fn handle_drop_event(app: &AppHandle, paths: &[PathBuf]) {
  let globset = glob::book();
  let books = paths
    .iter()
    .filter(|it| globset.is_match(it))
    .filter_map(|it| ActiveBook::new(app, it).ok())
    .collect_vec();

  if !books.is_empty() {
    let app = app.clone();
    spawn(async move {
      reader::open_many(&app, books)
        .await
        .into_err_dialog(&app);
    });
  }
}
