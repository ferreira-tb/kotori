use crate::book::ActiveBook;
use crate::reader;
use crate::utils::app::AppHandleExt;
use crate::utils::glob;
use crate::utils::result::ResultExt;
use itertools::Itertools;
use std::path::PathBuf;
use tauri::{async_runtime, AppHandle, DragDropEvent, WindowEvent};
use tracing::{info, trace};

pub fn on_main_window_event(app: &AppHandle) {
  let app = app.clone();
  let main_window = app.main_window();

  main_window.on_window_event(move |event| match event {
    WindowEvent::Destroyed => {
      info!("main window destroyed, exiting");
      app.exit(0);
    }
    WindowEvent::DragDrop(it) => {
      if let DragDropEvent::Dropped { paths, .. } = it {
        trace!(?paths, "dropped files");
        handle_drop_event(&app, &paths);
      }
    }
    _ => {}
  });
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
