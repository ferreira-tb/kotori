use crate::book::BookHandle;
use crate::database::DatabaseHandle;
use crate::reader::{Reader, WindowMap};
use crate::result::Result;
use crate::window::WindowKind;
use tauri::{AppHandle, Manager, State, WebviewWindow, Window, Wry};

pub struct Kotori {
  database_handle: DatabaseHandle,
  book_handle: BookHandle,
  reader: Reader,
}

impl Kotori {
  pub fn init(app: &AppHandle) -> Result<()> {
    let kotori = Self {
      database_handle: DatabaseHandle::new(app)?,
      book_handle: BookHandle::new(),
      reader: Reader::new(),
    };

    app.manage(kotori);

    Ok(())
  }
}

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
    self
      .get_webview_window(&label)
      .expect("main window MUST exist")
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
