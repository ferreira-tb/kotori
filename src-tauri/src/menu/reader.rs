use super::prelude::*;
use crate::{library, prelude::*, reader, utils::path};
use tauri::menu::MenuId;
use tauri_plugin_clipboard_manager::ClipboardExt;
use tokio::fs;

#[derive(Display, Debug, EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum Item {
  AddBookToLibrary,
  /// There's a [`tauri::menu::PredefinedMenuItem`] for this,
  /// but Linux doesn't support it.
  Close,
  CloseAll,
  CloseOthers,
  CopyBookPathToClipboard,
  OpenBookFolder,
}

impl Item {
  /// Unfortunately, we can't rely on [`strum::Display`] here.
  /// When a item on a menu is clicked, Tauri notifies all windows.
  /// Most of the time, this is fine, but as we have multiple reader windows,
  /// we need to know exactly which window the menu item was clicked on.
  pub fn to_menu_id(&self, window_id: u16) -> MenuId {
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
      debug!(menu_event = %item, reader_window = ctx.window_id);
      match item {
        Item::AddBookToLibrary => add_to_library(app, ctx.window_id),
        Item::Close => window.close().into_log(app),
        Item::CloseAll => close_all_reader_windows(app),
        Item::CloseOthers => close_other_reader_windows(app, ctx.window_id),
        Item::CopyBookPathToClipboard => copy_path_to_clipboard(app, ctx.window_id),
        Item::OpenBookFolder => open_book_folder(app, ctx.window_id),
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
  SubmenuBuilder::new(app, "File")
    .items(&[
      &menu_item!(app, Item::Close.to_menu_id(window_id), "Close", "Escape")?,
      &menu_item!(app, Item::CloseAll.to_menu_id(window_id), "Close all")?,
      &menu_item!(app, Item::CloseOthers.to_menu_id(window_id), "Close others")?,
    ])
    .separator()
    .items(&[
      &menu_item!(
        app,
        Item::CopyBookPathToClipboard.to_menu_id(window_id),
        "Copy book path"
      )?,
      &menu_item!(
        app,
        Item::OpenBookFolder.to_menu_id(window_id),
        "Open book folder"
      )?,
    ])
    .separator()
    .items(&[&menu_item!(
      app,
      Item::AddBookToLibrary.to_menu_id(window_id),
      "Add to library"
    )?])
    .build()
    .map_err(Into::into)
}

fn add_to_library(app: &AppHandle, window_id: u16) {
  let app = app.clone();
  async_runtime::spawn(async move {
    if let Some(path) = reader::get_book_path(&app, window_id).await {
      let result: Result<()> = try {
        library::save(app.clone(), path).await?;

        // Disable the menu item after adding the book to the library.
        let windows = app.reader_windows();
        let windows = windows.read().await;
        if let Some(window) = windows.get(&window_id) {
          window.set_menu_item_enabled(&Item::AddBookToLibrary, false)?;
        }
      };

      result.into_dialog(&app);
    };
  });
}

pub(super) fn close_all_reader_windows(app: &AppHandle) {
  let app = app.clone();
  async_runtime::spawn(async move {
    reader::close_all(&app).await.into_dialog(&app);
  });
}

fn close_other_reader_windows(app: &AppHandle, window_id: u16) {
  let app = app.clone();
  async_runtime::spawn(async move {
    reader::close_others(&app, window_id)
      .await
      .into_dialog(&app);
  });
}

fn copy_path_to_clipboard(app: &AppHandle, window_id: u16) {
  let app = app.clone();
  async_runtime::spawn(async move {
    let path = reader::get_book_path(&app, window_id)
      .await
      .and_then(|it| path::to_string(it).ok());

    if let Some(path) = path {
      app.clipboard().write_text(path).into_dialog(&app);
    }
  });
}

fn open_book_folder(app: &AppHandle, window_id: u16) {
  let app = app.clone();
  async_runtime::spawn(async move {
    let dir = reader::get_book_path(&app, window_id)
      .await
      .and_then(|it| it.parent().map(ToOwned::to_owned));

    if let Some(dir) = dir {
      if let Ok(true) = fs::try_exists(&dir).await {
        open::that_detached(dir).into_dialog(&app);
      }
    }
  });
}
