pub mod date {
  use chrono::Local;

  /// <https://docs.rs/chrono/latest/chrono/format/strftime/index.html>
  pub const TIMESTAMP: &str = "%F %T%.3f %:z";

  pub fn now() -> String {
    Local::now().format(TIMESTAMP).to_string()
  }
}

pub mod glob {
  use globset::{Glob, GlobBuilder, GlobSet, GlobSetBuilder};

  fn glob(glob: &str) -> Glob {
    GlobBuilder::new(glob)
      .case_insensitive(true)
      .build()
      .unwrap()
  }

  pub fn book() -> GlobSet {
    GlobSetBuilder::new()
      .add(glob("*.cbr"))
      .add(glob("*.cbz"))
      .build()
      .unwrap()
  }

  pub fn book_page() -> GlobSet {
    GlobSetBuilder::new()
      .add(glob("*.gif"))
      .add(glob("*.jpg"))
      .add(glob("*.jpeg"))
      .add(glob("*.png"))
      .add(glob("*.webp"))
      .build()
      .unwrap()
  }
}

pub mod webview {
  use crate::prelude::*;
  use tauri::WebviewUrl;

  pub fn dir<N: AsRef<str>>(app: &AppHandle, name: N) -> Result<PathBuf> {
    let name = name.as_ref();
    let path = app
      .path()
      .app_data_dir()?
      .join(format!("windows/{name}"));

    Ok(path)
  }

  pub fn url<N: AsRef<str>>(name: N) -> WebviewUrl {
    let name = name.as_ref();
    WebviewUrl::App(format!("src/windows/{name}/index.html",).into())
  }

  pub fn reader_dir(app: &AppHandle, id: u16) -> Result<PathBuf> {
    dir(app, format!("reader/{id}"))
  }

  pub fn reader_label(id: u16) -> String {
    format!("reader-{id}")
  }

  pub fn reader_url() -> WebviewUrl {
    url("reader")
  }
}
