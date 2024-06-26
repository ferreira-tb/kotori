use tauri::menu::AboutMetadataBuilder;
use tauri_plugin_dialog::{DialogExt, MessageDialogBuilder, MessageDialogKind};
use tauri_plugin_shell::ShellExt;
use tokio::sync::oneshot;

use crate::book::ActiveBook;
#[cfg(any(debug_assertions, feature = "devtools"))]
use crate::image::Orientation;
use crate::menu::prelude::*;
use crate::menu::Listener;
use crate::prelude::*;
use crate::window::ColorMode;
use crate::{library, menu_item_or_bail, reader, VERSION};

#[derive(Debug, Display, EnumString)]
pub enum Item {
  #[strum(serialize = "kt-app-about")]
  About,
  #[strum(serialize = "kt-app-add-to-library")]
  AddToLibrary,
  #[strum(serialize = "kt-app-color-mode-auto")]
  ColorModeAuto,
  #[strum(serialize = "kt-app-color-mode-dark")]
  ColorModeDark,
  #[strum(serialize = "kt-app-color-mode-light")]
  ColorModeLight,
  #[strum(serialize = "kt-app-discord")]
  Discord,
  #[strum(serialize = "kt-app-random-book")]
  RandomBook,
  #[strum(serialize = "kt-app-repository")]
  Repository,
  #[strum(serialize = "kt-app-scan-book-folders")]
  ScanBookFolders,
  #[strum(serialize = "kt-app-open-file")]
  OpenFile,

  #[cfg(any(debug_assertions, feature = "devtools"))]
  #[strum(serialize = "kt-app-add-mock-books-landscape")]
  AddMockBooksLandscape,
  #[cfg(any(debug_assertions, feature = "devtools"))]
  #[strum(serialize = "kt-app-add-mock-books-portrait")]
  AddMockBooksPortrait,
  #[cfg(any(debug_assertions, feature = "devtools"))]
  #[strum(serialize = "kt-app-remove-all-books")]
  RemoveAllBooks,
}

impl Listener for Item {
  fn execute(window: &Window, event: &MenuEvent) {
    let item = menu_item_or_bail!(event);
    let app = window.app_handle().clone();
    spawn(async move {
      match item {
        Item::About => {}
        Item::AddToLibrary => add_to_library_with_dialog(&app).await,
        Item::ColorModeAuto => set_color_mode(&app, ColorMode::Auto).await,
        Item::ColorModeDark => set_color_mode(&app, ColorMode::Dark).await,
        Item::ColorModeLight => set_color_mode(&app, ColorMode::Light).await,
        Item::Discord => open_discord(&app),
        Item::OpenFile => open_file(&app).await,
        Item::RandomBook => open_random_book(&app).await,
        Item::Repository => open_repository(&app),
        Item::ScanBookFolders => scan_book_folders(&app).await,

        #[cfg(any(debug_assertions, feature = "devtools"))]
        Item::AddMockBooksLandscape => add_mock_books(&app, Orientation::Landscape).await,
        #[cfg(any(debug_assertions, feature = "devtools"))]
        Item::AddMockBooksPortrait => add_mock_books(&app, Orientation::Portrait).await,
        #[cfg(any(debug_assertions, feature = "devtools"))]
        Item::RemoveAllBooks => remove_all_books(&app).await,
      }
    });
  }
}

pub fn build<M: Manager<Wry>>(app: &M) -> Result<Menu<Wry>> {
  let menu = Menu::new(app)?;
  menu.append(&file_menu(app)?)?;
  menu.append(&read_menu(app)?)?;
  menu.append(&view_menu(app)?)?;
  menu.append(&help_menu(app)?)?;

  #[cfg(any(debug_assertions, feature = "devtools"))]
  menu.append(&dev_menu(app)?)?;

  Ok(menu)
}

fn file_menu<M: Manager<Wry>>(app: &M) -> Result<Submenu<Wry>> {
  let mut menu = SubmenuBuilder::new(app, "File")
    .items(&[
      &menu_item!(app, Item::OpenFile, "Open file")?,
      &menu_item!(app, Item::AddToLibrary, "Add to library")?,
    ])
    .separator()
    .items(&[&menu_item!(
      app,
      Item::ScanBookFolders,
      "Scan book folders"
    )?]);

  if !cfg!(target_os = "linux") {
    menu = menu.separator().quit();
  }

  menu.build().map_err(Into::into)
}

fn read_menu<M: Manager<Wry>>(app: &M) -> Result<Submenu<Wry>> {
  SubmenuBuilder::new(app, "Read")
    .items(&[&menu_item!(app, Item::RandomBook, "Random book")?])
    .build()
    .map_err(Into::into)
}

fn view_menu<M: Manager<Wry>>(app: &M) -> Result<Submenu<Wry>> {
  let current_color_mode = ColorMode::get(app.app_handle())?;
  let color_mode = SubmenuBuilder::new(app, "Color mode")
    .items(&[
      &CheckMenuItemBuilder::with_id(Item::ColorModeAuto, "Auto")
        .checked(current_color_mode == ColorMode::Auto)
        .build(app)?,
      &CheckMenuItemBuilder::with_id(Item::ColorModeLight, "Light")
        .checked(current_color_mode == ColorMode::Light)
        .build(app)?,
      &CheckMenuItemBuilder::with_id(Item::ColorModeDark, "Dark")
        .checked(current_color_mode == ColorMode::Dark)
        .build(app)?,
    ])
    .build()?;

  SubmenuBuilder::new(app, "View")
    .items(&[&color_mode])
    .build()
    .map_err(Into::into)
}

fn help_menu<M: Manager<Wry>>(app: &M) -> Result<Submenu<Wry>> {
  let mut metadata = AboutMetadataBuilder::new()
    .name("Kotori".into())
    .version(VERSION.into())
    .copyright("Copyright © 2024 Andrew Ferreira".into());

  if !cfg!(target_os = "macos") {
    const LICENSE: &str = env!("CARGO_PKG_LICENSE");
    metadata = metadata.license(LICENSE.into());
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
fn dev_menu<M: Manager<Wry>>(app: &M) -> Result<Submenu<Wry>> {
  let mocks = SubmenuBuilder::new(app, "Mocks")
    .items(&[
      &menu_item!(app, Item::AddMockBooksPortrait, "Portrait")?,
      &menu_item!(app, Item::AddMockBooksLandscape, "Landscape")?,
    ])
    .build()?;

  SubmenuBuilder::new(app, "Developer")
    .items(&[&mocks])
    .separator()
    .items(&[&menu_item!(app, Item::RemoveAllBooks, "Remove all books")?])
    .build()
    .map_err(Into::into)
}

#[cfg(any(debug_assertions, feature = "devtools"))]
async fn add_mock_books(app: &AppHandle, orientation: Orientation) {
  library::add_mock_books(app, 15, 20, orientation)
    .await
    .dialog(app);
}

async fn add_to_library_with_dialog(app: &AppHandle) {
  library::add_with_dialog(app).await.dialog(app);
}

async fn open_file(app: &AppHandle) {
  crate::book::open_with_dialog(app)
    .await
    .dialog(app);
}

fn open_discord(app: &AppHandle) {
  app
    .shell()
    .open("https://discord.gg/aAje8qb49f", None)
    .dialog(app);
}

fn open_repository(app: &AppHandle) {
  const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
  app.shell().open(REPOSITORY, None).dialog(app);
}

async fn open_random_book(app: &AppHandle) {
  let result: Result<_> = try {
    if let Some(book) = ActiveBook::random(app).await? {
      reader::open_book(app, book).await?;
    }
  };

  result.dialog(app);
}

#[cfg(any(debug_assertions, feature = "devtools"))]
async fn remove_all_books(app: &AppHandle) {
  let (tx, rx) = oneshot::channel();
  let dialog = app.dialog().clone();

  let message = "All books will be removed. Are you sure?";
  MessageDialogBuilder::new(dialog, "Remove all books", message)
    .kind(MessageDialogKind::Warning)
    .ok_button_label("Clear")
    .cancel_button_label("Cancel")
    .show(move |response| {
      let _ = tx.send(response);
    });

  if let Ok(true) = rx.await {
    library::remove_all(app).await.dialog(app);
  }
}

async fn scan_book_folders(app: &AppHandle) {
  library::scan_book_folders(app).await.dialog(app);
}

async fn set_color_mode(app: &AppHandle, mode: ColorMode) {
  use tauri_plugin_manatsu::AppHandleExt as _;
  use tauri_plugin_window_state::{AppHandleExt as _, StateFlags};

  let (tx, rx) = oneshot::channel();
  let dialog = app.dialog().clone();

  let message = "Kotori must restart to apply the change. Do you want to continue?";
  MessageDialogBuilder::new(dialog, "Color mode", message)
    .kind(MessageDialogKind::Info)
    .ok_button_label("Confirm")
    .cancel_button_label("Cancel")
    .show(move |response| {
      let _ = tx.send(response);
    });

  if let Ok(true) = rx.await {
    let _ = mode.set(app);
    let _ = app.save_window_state(StateFlags::all());
    let _ = app.write_logs_to_disk();

    // Kotori may crash in dev mode after restarting.
    app.restart();
  }
}
