use super::prelude::*;
use crate::prelude::*;
use crate::{book, library, VERSION};
use tauri::menu::AboutMetadataBuilder;
use tauri_plugin_shell::ShellExt;

#[derive(Display, EnumString)]
enum Id {
  About,
  AddToLibrary,
  Repository,
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

  // Not supported on Linux.
  // https://docs.rs/tauri/2.0.0-beta/tauri/menu/struct.SubmenuBuilder.html#method.quit
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
    .name(Some("Kotori"))
    .version(Some(VERSION))
    .copyright(Some("Copyright © 2024 Andrew Ferreira"));

  if !cfg!(target_os = "macos") {
    metadata = metadata.license(Some("MIT"));
  }

  let metadata = metadata.build();
  let about = PredefinedMenuItem::about(app, Some("About"), Some(metadata))?;
  SubmenuBuilder::new(app, "Help")
    .items(&[&menu_item!(app, Id::Repository, "Repository")?])
    .item(&about)
    .build()
    .map_err(Into::into)
}

pub fn on_event<R>(app: &AppHandle) -> impl Fn(&Window<R>, MenuEvent)
where
  R: Runtime,
{
  let app = app.clone();
  move |_, event| {
    if let Ok(id) = Id::from_str(event.id.0.as_str()) {
      match id {
        Id::About => {}
        Id::AddToLibrary => add_to_library_from_dialog(&app),
        Id::Repository => open_repository(&app),
        Id::OpenBook => open_book_from_dialog(&app),
      }
    }
  }
}

fn add_to_library_from_dialog(app: &AppHandle) {
  let app = app.clone();
  async_runtime::spawn(async move {
    library::add_from_dialog(&app).await.ok();
  });
}

fn open_book_from_dialog(app: &AppHandle) {
  let app = app.clone();
  async_runtime::spawn(async move {
    book::open_from_dialog(&app).await.ok();
  });
}

fn open_repository(app: &AppHandle) {
  app
    .shell()
    .open("https://github.com/ferreira-tb/kotori", None)
    .ok();
}
