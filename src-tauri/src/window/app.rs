use crate::book::ActiveBook;
use crate::reader;
use crate::utils::glob;
use crate::utils::result::ResultExt;
use itertools::Itertools;
use std::path::PathBuf;
use tauri::menu::MenuEvent;
use tauri::DragDropEvent::Dropped;
use tauri::{async_runtime, AppHandle, Window, WindowEvent};
use tracing::{info, trace};

// Calling `on_menu_event` on a window will override previously registered event listeners.
// For this reason, all listeners must be registered inside a single call.
pub fn on_menu_event() -> impl Fn(&Window, MenuEvent) {
  use crate::menu::{self, context, Listener};
  move |window, event| {
    menu::app::Item::execute(window, &event);
    context::library::book::Item::execute(window, &event);
  }
}

pub fn on_window_event(app: &AppHandle) -> impl Fn(&WindowEvent) {
  let app = app.clone();
  move |event| match event {
    WindowEvent::Destroyed => {
      info!("main window destroyed, exiting");
      app.exit(0);
    }
    WindowEvent::DragDrop(Dropped { paths, .. }) => {
      trace!(?paths, "dropped files");
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
    .filter_map(|it| ActiveBook::new(it).ok())
    .collect_vec();

  if !books.is_empty() {
    let app = app.clone();
    async_runtime::spawn(async move {
      reader::open_many(&app, books)
        .await
        .into_dialog(&app);
    });
  }
}
