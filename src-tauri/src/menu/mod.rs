pub mod app;
pub mod context;
mod macros;
pub mod reader;

mod prelude {
  pub(super) use super::Listener;
  pub(super) use crate::menu_item;
  pub(super) use strum::{Display, EnumString};
  pub(super) use tauri::menu::{
    Menu, MenuBuilder, MenuEvent, MenuItemBuilder, PredefinedMenuItem, Submenu, SubmenuBuilder,
  };
}

use crate::prelude::*;
use tauri::menu::{Menu, MenuEvent, MenuId, MenuItemKind, Submenu};

pub trait Listener {
  type Context: Clone;

  fn execute(app: &AppHandle, window: &Window, event: &MenuEvent, ctx: Self::Context);

  fn on_event(app: AppHandle, ctx: Self::Context) -> impl Fn(&Window, MenuEvent) {
    move |window, event| {
      Self::execute(&app, window, &event, ctx.clone());
    }
  }
}

pub trait MenuExt {
  fn set_item_enabled(&self, id: &MenuId, enabled: bool) -> Result<()>;
}

impl<R: Runtime> MenuExt for Menu<R> {
  fn set_item_enabled(&self, id: &MenuId, enabled: bool) -> Result<()> {
    find_and_set_enabled(&self.items()?, id, enabled)
  }
}

impl<R: Runtime> MenuExt for Submenu<R> {
  fn set_item_enabled(&self, id: &MenuId, enabled: bool) -> Result<()> {
    if self.id() == id {
      debug!(menu_id = id.as_ref(), enabled);
      self.set_enabled(enabled).map_err(Into::into)
    } else {
      find_and_set_enabled(&self.items()?, id, enabled)
    }
  }
}

fn find_and_set_enabled<R>(items: &[MenuItemKind<R>], id: &MenuId, enabled: bool) -> Result<()>
where
  R: Runtime,
{
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
