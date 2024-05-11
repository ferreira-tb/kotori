use crate::book::ActiveBook;
use crate::prelude::*;

pub struct ReaderWindow {
  pub book: ActiveBook,
  pub(super) webview: WebviewWindow,
}

pub fn label(window_id: u16) -> String {
  format!("reader-{window_id}")
}

pub async fn get_window_id(app: &AppHandle, label: &str) -> Result<u16> {
  super::get_window_id_by_label(app, label)
    .await
    .ok_or_else(|| err!(WindowNotFound, "{label}"))
}
