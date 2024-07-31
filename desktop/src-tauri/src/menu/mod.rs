mod app;
pub mod context;
mod macros;
mod reader;

mod prelude {
  pub(super) use crate::menu::MenuExt;
  pub(super) use crate::{impl_deref_menu, menu_item_or_bail, mi, popup_context_menu};
  pub(super) use strum::{Display, EnumString};
  pub(super) use tauri::menu::{
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

// Currently, there's no way to set handlers from Rust.
// When it's possible, we should remove this trait.
// See: https://github.com/tauri-apps/tauri/pull/9380
pub trait Listener {
  fn execute(window: &Window, event: &MenuEvent);
}

trait MenuExt {
  /// Search for a menu item by its id and set its checked state.
  fn set_item_checked(&self, id: &MenuId, checked: bool) -> Result<()>;
  /// Search for a menu item by its id and set its enabled state.
  fn set_item_enabled(&self, id: &MenuId, enabled: bool) -> Result<()>;
}

impl MenuExt for Menu<Wry> {
  fn set_item_checked(&self, id: &MenuId, checked: bool) -> Result<()> {
    find_and_set_checked(&self.items()?, id, checked)
  }

  fn set_item_enabled(&self, id: &MenuId, enabled: bool) -> Result<()> {
    find_and_set_enabled(&self.items()?, id, enabled)
  }
}

impl MenuExt for Submenu<Wry> {
  fn set_item_checked(&self, id: &MenuId, checked: bool) -> Result<()> {
    find_and_set_checked(&self.items()?, id, checked)
  }

  fn set_item_enabled(&self, id: &MenuId, enabled: bool) -> Result<()> {
    if self.id() == id {
      #[cfg(feature = "tracing")]
      debug!(menu_id = id.as_ref(), enabled);

      self.set_enabled(enabled).map_err(Into::into)
    } else {
      find_and_set_enabled(&self.items()?, id, enabled)
    }
  }
}

fn find_and_set_checked(items: &[MenuItemKind<Wry>], id: &MenuId, checked: bool) -> Result<()> {
  for item in items {
    match item {
      MenuItemKind::Check(it) if it.id() == id => {
        #[cfg(feature = "tracing")]
        debug!(menu_id = id.as_ref(), checked);

        return it.set_checked(checked).map_err(Into::into);
      }
      MenuItemKind::Submenu(it) => it.set_item_checked(id, checked)?,
      _ => {}
    }
  }

  Ok(())
}

fn find_and_set_enabled(items: &[MenuItemKind<Wry>], id: &MenuId, enabled: bool) -> Result<()> {
  macro_rules! set_enabled {
    ($item:expr) => {
      if $item.id() == id {
        #[cfg(feature = "tracing")]
        debug!(menu_id = id.as_ref(), enabled);

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
