use super::prelude::*;
use crate::prelude::*;
use crate::utils::dialog;
use crate::{book, library, VERSION};
use tauri::menu::AboutMetadataBuilder;
use tauri_plugin_dialog::{DialogExt, MessageDialogBuilder, MessageDialogKind};
use tauri_plugin_shell::ShellExt;

#[derive(Debug, Display, EnumString)]
enum Id {
  #[strum(serialize = "kt-app-about")]
  About,
  #[strum(serialize = "kt-app-add-to-library")]
  AddToLibrary,
  #[strum(serialize = "kt-app-clear-library")]
  ClearLibrary,
  #[strum(serialize = "kt-app-discord")]
  Discord,
  #[strum(serialize = "kt-app-repository")]
  Repository,
  #[strum(serialize = "kt-app-open-book")]
  OpenBook,
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
    &menu_item!(app, Id::OpenBook, "Open book", "Ctrl+O")?,
    &menu_item!(app, Id::AddToLibrary, "Add to library", "Ctrl+Shift+A")?,
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
      &menu_item!(app, Id::Discord, "Discord")?,
      &menu_item!(app, Id::Repository, "Repository")?,
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
    .items(&[&menu_item!(app, Id::ClearLibrary, "Clear")?])
    .build()?;

  SubmenuBuilder::new(app, "Developer")
    .items(&[&library])
    .build()
    .map_err(Into::into)
}

pub fn on_event<R>(app: &AppHandle) -> impl Fn(&Window<R>, MenuEvent)
where
  R: Runtime,
{
  let app = app.clone();
  move |_, event| {
    if let Ok(id) = Id::try_from(event.id().as_ref()) {
      match id {
        Id::About => {}
        Id::AddToLibrary => add_to_library_from_dialog(&app),
        Id::ClearLibrary => clear_library(&app),
        Id::Discord => open_discord(&app),
        Id::Repository => open_repository(&app),
        Id::OpenBook => open_book_from_dialog(&app),
      }
    }
  }
}

fn add_to_library_from_dialog(app: &AppHandle) {
  let app = app.clone();
  async_runtime::spawn(async move {
    if let Err(error) = library::add_from_dialog(&app).await {
      error!(%error);
      dialog::show_error(&app, error);
    }
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
      if let Err(error) = library::remove_all(&app).await {
        error!(%error);
        dialog::show_error(&app, error);
      }
    }
  });
}

fn open_book_from_dialog(app: &AppHandle) {
  let app = app.clone();
  async_runtime::spawn(async move {
    if let Err(error) = book::open_from_dialog(&app).await {
      error!(%error);
      dialog::show_error(&app, error);
    }
  });
}

fn open_discord(app: &AppHandle) {
  let _ = app
    .shell()
    .open("https://discord.gg/aAje8qb49f", None);
}

fn open_repository(app: &AppHandle) {
  let _ = app
    .shell()
    .open("https://github.com/ferreira-tb/kotori", None);
}
