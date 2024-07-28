mod app;
pub mod context;
mod macros;
mod reader;

mod prelude {
  pub use crate::mi;
  pub use strum::{Display, EnumString};
  pub use tauri::menu::{
    CheckMenuItemBuilder, Menu, MenuBuilder, MenuEvent, MenuId, MenuItemKind, PredefinedMenuItem,
    Submenu, SubmenuBuilder,
  };
}

use crate::prelude::*;
pub use app::{AppMenu, Item as AppMenuItem};
pub use context::library_book::Item as LibraryBookContextMenuItem;
pub use context::reader_page::Item as ReaderPageContextMenuItem;
use prelude::*;
pub use reader::{Item as ReaderMenuItem, ReaderMenu};

pub trait Listener {
  fn execute(window: &Window, event: &MenuEvent);
}

pub trait MenuExt {
  fn set_item_enabled(&self, id: &MenuId, enabled: bool) -> Result<()>;
}

impl MenuExt for Menu<Wry> {
  fn set_item_enabled(&self, id: &MenuId, enabled: bool) -> Result<()> {
    find_and_set_enabled(&self.items()?, id, enabled)
  }
}

impl MenuExt for Submenu<Wry> {
  fn set_item_enabled(&self, id: &MenuId, enabled: bool) -> Result<()> {
    if self.id() == id {
      debug!(menu_id = id.as_ref(), enabled);
      self.set_enabled(enabled).map_err(Into::into)
    } else {
      find_and_set_enabled(&self.items()?, id, enabled)
    }
  }
}

fn find_and_set_enabled(items: &[MenuItemKind<Wry>], id: &MenuId, enabled: bool) -> Result<()> {
  macro_rules! set_enabled {
    ($item:expr) => {
      if $item.id() == id {
        tracing::debug!(menu_id = id.as_ref(), enabled);
        return $item.set_enabled(enabled).map_err(Into::into);
      }
    };
  }

  for item in items {
    match item {
      MenuItemKind::Check(it) => set_enabled!(it),
      MenuItemKind::Icon(it) => set_enabled!(it),
      MenuItemKind::MenuItem(it) => set_enabled!(it),
      MenuItemKind::Submenu(it) => it.set_item_enabled(id, enabled)?,
      MenuItemKind::Predefined(_) => {}
    }
  }

  Ok(())
}