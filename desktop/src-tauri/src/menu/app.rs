use crate::book::ActiveBook;
#[cfg(feature = "devtools")]
use crate::image::mock::Orientation;
use crate::menu::prelude::*;
use crate::menu::Listener;
use crate::prelude::*;
use crate::window::ColorMode;
use crate::{library, reader, VERSION};
use tauri::menu::AboutMetadataBuilder;
use tokio::sync::oneshot;

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

  #[cfg(feature = "devtools")]
  #[strum(serialize = "kt-app-add-mock-books-landscape")]
  AddMockBooksLandscape,
  #[cfg(feature = "devtools")]
  #[strum(serialize = "kt-app-add-mock-books-portrait")]
  AddMockBooksPortrait,
  #[cfg(feature = "devtools")]
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

        #[cfg(feature = "devtools")]
        Item::AddMockBooksLandscape => add_mock_books(&app, Orientation::Landscape).await,
        #[cfg(feature = "devtools")]
        Item::AddMockBooksPortrait => add_mock_books(&app, Orientation::Portrait).await,
        #[cfg(feature = "devtools")]
        Item::RemoveAllBooks => remove_all_books(&app).await,
      }
    });
  }
}

pub struct AppMenu;

impl AppMenu {
  pub fn build<M: Manager<Wry>>(app: &M) -> Result<Menu<Wry>> {
    let menu = Menu::new(app)?;
    menu.append(&*FileMenu::new(app)?)?;
    menu.append(&*ReadMenu::new(app)?)?;
    menu.append(&*ViewMenu::new(app)?)?;
    menu.append(&*HelpMenu::new(app)?)?;

    #[cfg(feature = "devtools")]
    menu.append(&*DevMenu::new(app)?)?;

    Ok(menu)
  }
}

struct FileMenu(Submenu<Wry>);

impl FileMenu {
  fn new<M: Manager<Wry>>(app: &M) -> Result<Self> {
    let mut menu = SubmenuBuilder::new(app, "File")
      .items(&[
        &mi!(app, OpenFile, "Open file")?,
        &mi!(app, AddToLibrary, "Add to library")?,
      ])
      .separator()
      .items(&[&mi!(app, ScanBookFolders, "Scan book folders")?]);

    if !cfg!(target_os = "linux") {
      menu = menu.separator().quit();
    }

    menu.build().map(Self).map_err(Into::into)
  }
}

struct ReadMenu(Submenu<Wry>);

impl ReadMenu {
  fn new<M: Manager<Wry>>(app: &M) -> Result<Self> {
    SubmenuBuilder::new(app, "Read")
      .items(&[&mi!(app, RandomBook, "Random book")?])
      .build()
      .map(Self)
      .map_err(Into::into)
  }
}

struct ViewMenu(Submenu<Wry>);

impl ViewMenu {
  fn new<M: Manager<Wry>>(app: &M) -> Result<Self> {
    SubmenuBuilder::new(app, "View")
      .items(&[&ViewMenu::color_mode(app)?])
      .build()
      .map(Self)
      .map_err(Into::into)
  }

  fn color_mode<M: Manager<Wry>>(app: &M) -> Result<Submenu<Wry>> {
    let current_color_mode = ColorMode::get(app.app_handle())?;
    SubmenuBuilder::new(app, "Color mode")
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
      .build()
      .map_err(Into::into)
  }
}

struct HelpMenu(Submenu<Wry>);

impl HelpMenu {
  fn new<M: Manager<Wry>>(app: &M) -> Result<Self> {
    SubmenuBuilder::new(app, "Help")
      .items(&[
        &mi!(app, Discord, "Discord")?,
        &mi!(app, Repository, "Repository")?,
      ])
      .item(&Self::about(app)?)
      .build()
      .map(Self)
      .map_err(Into::into)
  }

  fn about<M: Manager<Wry>>(app: &M) -> Result<PredefinedMenuItem<Wry>> {
    let mut metadata = AboutMetadataBuilder::new()
      .name("Kotori".into())
      .version(VERSION.into())
      .copyright("Copyright © 2024 Andrew Ferreira".into());

    if !cfg!(target_os = "macos") {
      const LICENSE: &str = env!("CARGO_PKG_LICENSE");
      metadata = metadata.license(LICENSE.into());
    }

    let metadata = Some(metadata.build());
    PredefinedMenuItem::about(app, "About".into(), metadata).map_err(Into::into)
  }
}

#[cfg(feature = "devtools")]
struct DevMenu(Submenu<Wry>);

#[cfg(feature = "devtools")]
impl DevMenu {
  fn new<M: Manager<Wry>>(app: &M) -> Result<Self> {
    let mocks = SubmenuBuilder::new(app, "Mocks")
      .items(&[
        &mi!(app, AddMockBooksPortrait, "Portrait")?,
        &mi!(app, AddMockBooksLandscape, "Landscape")?,
      ])
      .build()?;

    SubmenuBuilder::new(app, "Developer")
      .items(&[&mocks])
      .separator()
      .items(&[&mi!(app, RemoveAllBooks, "Remove all books")?])
      .build()
      .map(Self)
      .map_err(Into::into)
  }
}

impl_deref_menu!(FileMenu, ReadMenu, ViewMenu, HelpMenu);

#[cfg(feature = "devtools")]
impl_deref_menu!(DevMenu);

#[cfg(feature = "devtools")]
async fn add_mock_books(app: &AppHandle, orientation: Orientation) {
  library::add_mock_books(app, 15, 20, orientation)
    .await
    .into_err_dialog(app);
}

async fn add_to_library_with_dialog(app: &AppHandle) {
  library::add_with_dialog(app)
    .await
    .into_err_dialog(app);
}

async fn open_file(app: &AppHandle) {
  crate::book::open_with_dialog(app)
    .await
    .into_err_dialog(app);
}

fn open_discord(app: &AppHandle) {
  use tauri_plugin_shell::ShellExt;

  app
    .shell()
    .open("https://discord.gg/aAje8qb49f", None)
    .map_err(Into::into)
    .into_err_dialog(app);
}

fn open_repository(app: &AppHandle) {
  use tauri_plugin_shell::ShellExt;

  const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
  app
    .shell()
    .open(REPOSITORY, None)
    .map_err(Into::into)
    .into_err_dialog(app);
}

async fn open_random_book(app: &AppHandle) {
  let result: Result<_> = try {
    if let Some(book) = ActiveBook::random(app).await? {
      reader::open_book(app, book).await?;
    }
  };

  result.into_err_dialog(app);
}

#[cfg(feature = "devtools")]
async fn remove_all_books(app: &AppHandle) {
  use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

  let (tx, rx) = oneshot::channel();
  app
    .dialog()
    .message("All books will be removed. Are you sure?")
    .title("Remove all books")
    .kind(MessageDialogKind::Warning)
    .ok_button_label("Clear")
    .cancel_button_label("Cancel")
    .show(move |response| {
      let _ = tx.send(response);
    });

  if let Ok(true) = rx.await {
    library::remove_all(app)
      .await
      .into_err_dialog(app);
  }
}

async fn scan_book_folders(app: &AppHandle) {
  library::scan_book_folders(app)
    .await
    .into_err_dialog(app);
}

async fn set_color_mode(app: &AppHandle, mode: ColorMode) {
  use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
  use tauri_plugin_manatsu::AppHandleExt as _;
  use tauri_plugin_window_state::{AppHandleExt as _, StateFlags};

  let (tx, rx) = oneshot::channel();
  app
    .dialog()
    .message("Kotori must restart to apply the change. Do you want to continue?")
    .title("Color mode")
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

    // Kotori will crash in dev mode after restarting.
    app.restart();
  }
}
