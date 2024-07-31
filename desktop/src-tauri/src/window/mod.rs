pub mod app;
mod reader;

use crate::prelude::*;
use crate::reader::WindowMap;
use crate::Kotori;
pub use reader::ReaderWindow;
use strum::{Display, EnumIs, EnumString};
use tauri::{EventTarget, WebviewUrl};

#[derive(Debug, Display, EnumIs)]
#[strum(serialize_all = "kebab-case")]
pub enum WindowKind {
  Main,
  #[strum(to_string = "reader-{0}")]
  Reader(u16),
}

impl WindowKind {
  pub fn label(&self) -> String {
    self.to_string()
  }

  fn data_dir(&self, app: &AppHandle) -> Result<PathBuf> {
    let label = self.label();
    app
      .path()
      .app_local_data_dir()
      .map(|it| it.join(format!("windows/{label}")))
      .map_err(Into::into)
  }

  fn url(&self) -> WebviewUrl {
    let name = match self {
      Self::Main => "main",
      Self::Reader { .. } => "reader",
    };

    WebviewUrl::App(format!("src/windows/{name}/index.html",).into())
  }
}

impl From<WindowKind> for EventTarget {
  fn from(kind: WindowKind) -> Self {
    let label = kind.label();
    EventTarget::WebviewWindow { label }
  }
}

pub trait WindowManager: Manager<Wry> {
  fn get_focused_window(&self) -> Option<WebviewWindow> {
    self
      .webview_windows()
      .into_values()
      .find(|it| it.is_focused().unwrap_or(false))
  }

  fn main_window(&self) -> WebviewWindow {
    let label = WindowKind::Main.label();
    self.get_webview_window(&label).unwrap()
  }

  fn reader_windows(&self) -> WindowMap {
    self.state::<Kotori>().reader.windows()
  }
}

impl WindowManager for AppHandle {}
impl WindowManager for WebviewWindow {}
impl WindowManager for Window {}

pub trait WindowExt {
  /// Same as [`WebviewWindow::set_focus`], but unminimize the window before focusing.
  fn set_foreground_focus(&self) -> Result<()>;
}

impl WindowExt for WebviewWindow {
  fn set_foreground_focus(&self) -> Result<()> {
    if self.is_minimized()? {
      self.unminimize()?;
    }

    self.set_focus().map_err(Into::into)
  }
}

#[derive(Debug, Default, Display, EnumString, Eq, PartialEq)]
#[strum(serialize_all = "lowercase")]
pub enum ColorMode {
  #[default]
  Auto,
  Dark,
  Light,
}

impl ColorMode {
  const KEY: &str = "colorMode";

  pub fn get(app: &AppHandle) -> Result<Self> {
    use tauri_plugin_pinia::ManagerExt;

    app
      .with_store("config", |store| {
        let mode = store
          .get(Self::KEY)
          .and_then(|it| it.as_str())
          .and_then(|it| ColorMode::try_from(it).ok())
          .unwrap_or_default();

        Ok(mode)
      })
      .map_err(Into::into)
  }

  pub fn set(&self, app: &AppHandle) -> Result<()> {
    use tauri_plugin_pinia::ManagerExt;

    app
      .with_store("config", |store| {
        let mode = self.to_string();
        store.set(Self::KEY, mode.into())
      })
      .map_err(Into::into)
  }
}

impl From<ColorMode> for Option<tauri::Theme> {
  fn from(value: ColorMode) -> Option<tauri::Theme> {
    match value {
      ColorMode::Auto => None,
      ColorMode::Dark => Some(tauri::Theme::Dark),
      ColorMode::Light => Some(tauri::Theme::Light),
    }
  }
}
