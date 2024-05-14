use super::prelude::*;
use super::reader::close_all_reader_windows;
use crate::{book, library, menu_item_or_bail, prelude::*, VERSION};
use tauri::menu::AboutMetadataBuilder;
use tauri_plugin_dialog::{DialogExt, MessageDialogBuilder, MessageDialogKind};
use tauri_plugin_shell::ShellExt;

#[derive(Debug, Display, EnumString)]
pub enum Item {
  #[strum(serialize = "kt-app-about")]
  About,
  #[strum(serialize = "kt-app-add-to-library")]
  AddToLibrary,
  #[strum(serialize = "kt-app-clear-library")]
  ClearLibrary,
  #[strum(serialize = "kt-app-close-all-reader-windows")]
  CloseAllReaderWindows,
  #[strum(serialize = "kt-app-discord")]
  Discord,
  #[strum(serialize = "kt-app-repository")]
  Repository,
  #[strum(serialize = "kt-app-open-book")]
  OpenBook,
}

impl Listener for Item {
  type Context = ();

  fn execute(app: &AppHandle, _: &Window, event: &MenuEvent, (): Self::Context) {
    let item = menu_item_or_bail!(event);
    match item {
      Item::About => {}
      Item::AddToLibrary => add_to_library_from_dialog(app),
      Item::ClearLibrary => clear_library(app),
      Item::CloseAllReaderWindows => close_all_reader_windows(app),
      Item::Discord => open_discord(app),
      Item::Repository => open_repository(app),
      Item::OpenBook => open_book_from_dialog(app),
    }
  }
}

pub fn build<M, R>(app: &M) -> Result<Menu<R>>
where
  R: Runtime,
  M: Manager<R>,
{
  let menu = Menu::new(app)?;
  menu.append(&file_menu(app)?)?;
  menu.append(&help_menu(app)?)?;

  #[cfg(any(debug_assertions, feature = "devtools"))]
  menu.append(&dev_menu(app)?)?;

  Ok(menu)
}

fn file_menu<M, R>(app: &M) -> Result<Submenu<R>>
where
  R: Runtime,
  M: Manager<R>,
{
  let mut menu = SubmenuBuilder::new(app, "File").items(&[
    &menu_item!(app, Item::OpenBook, "Open book", "Ctrl+O")?,
    &menu_item!(app, Item::AddToLibrary, "Add to library", "Ctrl+Shift+A")?,
  ]);

  if !cfg!(target_os = "linux") {
    menu = menu.separator().quit();
  }

  menu.build().map_err(Into::into)
}

fn help_menu<M, R>(app: &M) -> Result<Submenu<R>>
where
  R: Runtime,
  M: Manager<R>,
{
  let mut metadata = AboutMetadataBuilder::new()
    .name("Kotori".into())
    .version(VERSION.into())
    .copyright("Copyright © 2024 Andrew Ferreira".into());

  if !cfg!(target_os = "macos") {
    metadata = metadata.license("MIT".into());
  }

  let metadata = metadata.build();
  let about = PredefinedMenuItem::about(app, "About".into(), metadata.into())?;
  SubmenuBuilder::new(app, "Help")
    .items(&[
      &menu_item!(app, Item::Discord, "Discord")?,
      &menu_item!(app, Item::Repository, "Repository")?,
    ])
    .item(&about)
    .build()
    .map_err(Into::into)
}

#[cfg(any(debug_assertions, feature = "devtools"))]
fn dev_menu<M, R>(app: &M) -> Result<Submenu<R>>
where
  R: Runtime,
  M: Manager<R>,
{
  let library = SubmenuBuilder::new(app, "Library")
    .items(&[&menu_item!(app, Item::ClearLibrary, "Clear")?])
    .build()?;

  let reader = SubmenuBuilder::new(app, "Reader")
    .items(&[&menu_item!(app, Item::CloseAllReaderWindows, "Close all")?])
    .build()?;

  SubmenuBuilder::new(app, "Developer")
    .items(&[&library, &reader])
    .build()
    .map_err(Into::into)
}

fn add_to_library_from_dialog(app: &AppHandle) {
  let app = app.clone();
  async_runtime::spawn(async move {
    let _ = library::add_from_dialog(&app)
      .await
      .show_dialog_on_error(&app);
  });
}

fn clear_library(app: &AppHandle) {
  let app = app.clone();
  async_runtime::spawn(async move {
    let (tx, rx) = oneshot::channel();
    let dialog = app.dialog().clone();

    let message = "All books will be removed.";
    MessageDialogBuilder::new(dialog, "Clear library", message)
      .kind(MessageDialogKind::Warning)
      .ok_button_label("Clear")
      .cancel_button_label("Cancel")
      .show(move |response| {
        let _ = tx.send(response);
      });

    if let Ok(true) = rx.await {
      let _ = library::remove_all(&app)
        .await
        .show_dialog_on_error(&app);
    }
  });
}

fn open_book_from_dialog(app: &AppHandle) {
  let app = app.clone();
  async_runtime::spawn(async move {
    let _ = book::open_from_dialog(&app)
      .await
      .show_dialog_on_error(&app);
  });
}

fn open_discord(app: &AppHandle) {
  let _ = app
    .shell()
    .open("https://discord.gg/aAje8qb49f", None)
    .show_dialog_on_error(app);
}

fn open_repository(app: &AppHandle) {
  let _ = app
    .shell()
    .open("https://github.com/ferreira-tb/kotori", None)
    .show_dialog_on_error(app);
}
