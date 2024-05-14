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
use tauri::menu::MenuEvent;

pub trait Listener {
  type Context: Clone;

  fn execute(app: &AppHandle, window: &Window, event: &MenuEvent, ctx: Self::Context);

  fn on_event(app: AppHandle, ctx: Self::Context) -> impl Fn(&Window, MenuEvent) {
    move |window, event| {
      Self::execute(&app, window, &event, ctx.clone());
    }
  }
}
