pub mod context;
mod macros;
pub mod main;

mod prelude {
  pub(super) use crate::menu_item;
  pub(super) use std::str::FromStr;
  pub(super) use strum::{Display, EnumString};
  pub(super) use tauri::menu::{
    Menu, MenuBuilder, MenuEvent, MenuItemBuilder, PredefinedMenuItem, Submenu, SubmenuBuilder,
  };
}
