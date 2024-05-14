use super::prelude::*;
use crate::{prelude::*, reader};
use tauri::menu::MenuId;

#[derive(Display, Debug, EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum Item {
  /// There's a [`tauri::menu::PredefinedMenuItem`] for this,
  /// but Linux doesn't support it.
  Close,
  CloseAll,
  CloseOthers,
  CopyFilePath,
  RevealInExplorer,
}

impl Item {
  /// Unfortunately, we can't rely on [`strum::Display`] here.
  /// When a item on a menu is clicked, Tauri notifies all windows.
  /// Most of the time, this is fine, but as we have multiple reader windows,
  /// we need to know exactly which window the menu item was clicked on.
  fn to_menu_id(&self, window_id: u16) -> MenuId {
    let prefix = Self::prefix(window_id);
    MenuId::new(format!("{prefix}{self}"))
  }

  fn from_menu_id(menu_id: &MenuId, window_id: u16) -> Option<Self> {
    menu_id
      .as_ref()
      .strip_prefix(&Self::prefix(window_id))
      .and_then(|it| Self::try_from(it).ok())
  }

  fn prefix(window_id: u16) -> String {
    format!("kt-reader-{window_id}-")
  }
}

impl Listener for Item {
  type Context = Context;

  fn execute(app: &AppHandle, window: &Window, event: &MenuEvent, ctx: Self::Context) {
    if let Some(item) = Self::from_menu_id(event.id(), ctx.window_id) {
      debug!(menu_event = %item);
      #[allow(clippy::match_same_arms)]
      match item {
        Item::Close => {
          let _ = window.close();
        }
        Item::CloseAll => close_all_reader_windows(app),
        Item::CloseOthers => close_other_reader_windows(app, ctx.window_id),
        Item::CopyFilePath => {}
        Item::RevealInExplorer => {}
      }
    };
  }
}

#[derive(Clone)]
pub struct Context {
  pub window_id: u16,
}

pub fn build<M, R>(app: &M, window_id: u16) -> Result<Menu<R>>
where
  R: Runtime,
  M: Manager<R>,
{
  let menu = Menu::new(app)?;
  menu.append(&file_menu(app, window_id)?)?;

  Ok(menu)
}

fn file_menu<M, R>(app: &M, window_id: u16) -> Result<Submenu<R>>
where
  R: Runtime,
  M: Manager<R>,
{
  let menu = SubmenuBuilder::new(app, "File")
    .items(&[
      &menu_item!(app, Item::Close.to_menu_id(window_id), "Close", "Escape")?,
      &menu_item!(app, Item::CloseAll.to_menu_id(window_id), "Close all")?,
      &menu_item!(app, Item::CloseOthers.to_menu_id(window_id), "Close others")?,
    ])
    .separator()
    .items(&[
      &menu_item!(
        app,
        Item::CopyFilePath.to_menu_id(window_id),
        "Copy file path"
      )?,
      &menu_item!(
        app,
        Item::RevealInExplorer.to_menu_id(window_id),
        "Reveal in explorer"
      )?,
    ]);

  menu.build().map_err(Into::into)
}

pub(super) fn close_all_reader_windows(app: &AppHandle) {
  let app = app.clone();
  async_runtime::spawn(async move {
    let _ = reader::close_all(&app)
      .await
      .show_dialog_on_error(&app);
  });
}

fn close_other_reader_windows(app: &AppHandle, window_id: u16) {
  let app = app.clone();
  async_runtime::spawn(async move {
    let _ = reader::close_others(&app, window_id)
      .await
      .show_dialog_on_error(&app);
  });
}
