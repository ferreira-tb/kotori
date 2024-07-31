use crate::error::Error;
use std::fmt;
use tauri::AppHandle;
use tauri_plugin_manatsu::Log;
use tokio::sync::oneshot;

pub type Result<T> = std::result::Result<T, Error>;
pub type BoxResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub type TxResult<T> = oneshot::Sender<Result<T>>;

// TODO: We should use the context provided by the `Error` type to show better error messages.
pub trait ResultExt<T> {
  /// Create an error log, consuming `self`, and discarding the success value, if any.
  fn into_err_log(self, app: &AppHandle);

  /// Show an error dialog, consuming `self`, and discarding the success value, if any.
  fn into_err_dialog(self, app: &AppHandle);
}

impl<T> ResultExt<T> for Result<T> {
  fn into_err_log(self, app: &AppHandle) {
    if let Err(err) = self {
      let message = err.to_string();
      let _ = Log::new("Error", message).save(app);
    }
  }

  fn into_err_dialog(self, app: &AppHandle) {
    if let Err(err) = &self {
      show_error_dialog(app, err);
      self.into_err_log(app);
    }
  }
}

fn show_error_dialog(app: &AppHandle, error: impl fmt::Display) {
  use tauri_plugin_dialog::{DialogExt, MessageDialogBuilder, MessageDialogKind};

  let dialog = app.dialog().clone();
  MessageDialogBuilder::new(dialog, "Error", error.to_string())
    .kind(MessageDialogKind::Error)
    .show(|_| {});
}
