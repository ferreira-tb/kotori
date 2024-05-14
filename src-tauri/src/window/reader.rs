use super::WindowKind;
use crate::book::ActiveBook;
use crate::menu::reader::{Context, Item};
use crate::menu::{self, Listener};
use crate::prelude::*;
use std::sync::atomic::{self, AtomicU16};
use tauri::{WebviewWindowBuilder, WindowEvent};

static WINDOW_ID: AtomicU16 = AtomicU16::new(0);

pub struct ReaderWindow {
  pub id: u16,
  pub book: ActiveBook,
  pub webview: WebviewWindow,
}

impl ReaderWindow {
  pub fn open(app: &AppHandle, book: ActiveBook) -> Result<(u16, Self)> {
    let id = WINDOW_ID.fetch_add(1, atomic::Ordering::SeqCst);
    let script = format!("window.KOTORI = {{ readerWindowId: {id} }}");
    trace!(%script);

    let kind = WindowKind::Reader(id);
    let window = WebviewWindowBuilder::new(app, kind.label(), kind.url())
      .initialization_script(&script)
      .data_directory(kind.data_dir(app)?)
      .title(book.title.to_string())
      .min_inner_size(800.0, 600.0)
      .resizable(true)
      .maximizable(true)
      .minimizable(true)
      .visible(false)
      .build()
      .map(|webview| Self { id, book, webview })?;

    on_window_event(app, &window.webview, id);

    let menu = menu::reader::build(app, id)?;
    window.webview.set_menu(menu)?;

    // This menu must be hidden by default.
    window.webview.hide_menu()?;

    let ctx = Context { window_id: id };
    window
      .webview
      .on_menu_event(Item::on_event(app.clone(), ctx));

    Ok((id, window))
  }
}

fn on_window_event(app: &AppHandle, webview: &WebviewWindow, window_id: u16) {
  let app = app.clone();
  webview.on_window_event(move |event| {
    if matches!(event, WindowEvent::CloseRequested { .. }) {
      info!("close requested for reader window {window_id}");
      let app = app.clone();
      async_runtime::spawn(async move {
        let windows = app.reader_windows();
        let mut windows = windows.write().await;
        windows.shift_remove(&window_id);

        info!("reader window {window_id} closed");
      });
    }
  });
}
