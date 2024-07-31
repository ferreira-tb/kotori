use crate::book::BookHandle;
use crate::database::DatabaseHandle;
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
}

impl ManagerExt for AppHandle {}
impl ManagerExt for WebviewWindow {}
impl ManagerExt for Window {}
