use super::WindowMap;
use crate::book::ActiveBook;
use crate::prelude::*;

pub struct ReaderWindow {
  pub book: ActiveBook,
  pub(super) webview: WebviewWindow,
}

pub async fn get_windows(app: &AppHandle) -> WindowMap {
  let kotori = app.state::<Kotori>();
  let reader = kotori.reader.read().await;
  reader.windows()
}

pub async fn get_window_id(app: &AppHandle, window: &WebviewWindow) -> Result<u16> {
  let kotori = app.state::<Kotori>();
  let reader = kotori.reader.read().await;

  let label = window.label();
  reader
    .get_window_id_by_label(label)
    .await
    .ok_or_else(|| err!(WindowNotFound, "{label}"))
}
