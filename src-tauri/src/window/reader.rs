use super::WindowKind;
use crate::book::ActiveBook;
use crate::prelude::*;
use std::sync::atomic::{self, AtomicU16};
use tauri::WebviewWindowBuilder;

static WINDOW_ID: AtomicU16 = AtomicU16::new(0);

pub struct ReaderWindow {
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
      .map(|webview| Self { book, webview })?;

    Ok((id, window))
  }
}
