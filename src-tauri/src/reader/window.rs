use super::WindowMap;
use crate::book::ActiveBook;
use crate::prelude::*;

pub struct ReaderWindow {
  pub book: ActiveBook,
  pub(super) webview: WebviewWindow,
}

pub fn label(window_id: u16) -> String {
  format!("reader-{window_id}")
}

pub async fn get_windows(app: &AppHandle) -> WindowMap {
  let kotori = app.kotori();
  kotori.reader.windows()
}

pub async fn get_window_id(app: &AppHandle, label: &str) -> Result<u16> {
  let kotori = app.kotori();
  kotori
    .reader
    .get_window_id_by_label(label)
    .await
    .ok_or_else(|| err!(WindowNotFound, "{label}"))
}
