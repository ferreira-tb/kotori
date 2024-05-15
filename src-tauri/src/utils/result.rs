use super::dialog;
use std::error::Error;
use tauri::{async_runtime, AppHandle};
use tauri_plugin_manatsu::Log;

pub trait ResultExt<T, E: Error> {
  /// Saves an error log, consuming `self`, and discarding the success value, if any.
  fn into_log(self, app: &AppHandle);

  /// Shows an error dialog, consuming `self`, and discarding the success value, if any.
  fn into_dialog(self, app: &AppHandle);
}

impl<T, E: Error> ResultExt<T, E> for Result<T, E> {
  fn into_log(self, app: &AppHandle) {
    if let Err(error) = self {
      tracing::error!(%error);

      let app = app.clone();
      let message = error.to_string();
      async_runtime::spawn(async move {
        let _ = Log::new("Error", message)
          .save(&app)
          .await
          .inspect_err(|error| tracing::error!(%error));
      });
    }
  }

  fn into_dialog(self, app: &AppHandle) {
    if let Err(error) = &self {
      dialog::show_error(app, error);
      self.into_log(app);
    }
  }
}
