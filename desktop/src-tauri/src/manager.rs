use crate::book::BookHandle;
use crate::database::DatabaseHandle;
use crate::reader::WindowMap;
use crate::window::WindowKind;
use crate::Kotori;
use tauri::{AppHandle, Manager, State, WebviewWindow, Window, Wry};

pub trait ManagerExt: Manager<Wry> {
  fn kotori(&self) -> State<Kotori> {
    self.state::<Kotori>()
  }

  fn book_handle(&self) -> BookHandle {
    self.kotori().book_handle.clone()
  }

  fn database_handle(&self) -> DatabaseHandle {
    self.kotori().database_handle.clone()
  }

  fn main_window(&self) -> WebviewWindow {
    let label = WindowKind::Main.label();
    self.get_webview_window(&label).unwrap()
  }

  fn reader_windows(&self) -> WindowMap {
    self.state::<Kotori>().reader.windows()
  }

  fn get_focused_window(&self) -> Option<WebviewWindow> {
    self
      .webview_windows()
      .into_values()
      .find(|it| it.is_focused().unwrap_or(false))
  }
}

impl ManagerExt for AppHandle {}
impl ManagerExt for WebviewWindow {}
impl ManagerExt for Window {}
