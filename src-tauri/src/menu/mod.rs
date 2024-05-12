pub mod app;
pub mod context;
mod macros;

mod prelude {
  pub(super) use crate::menu_item;
  pub(super) use strum::{Display, EnumString};
  pub(super) use tauri::menu::{
    Menu, MenuBuilder, MenuEvent, MenuItemBuilder, PredefinedMenuItem, Submenu, SubmenuBuilder,
  };
}
