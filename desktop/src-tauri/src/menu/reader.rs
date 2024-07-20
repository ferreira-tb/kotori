use crate::book::ActiveBook;
use crate::menu::prelude::*;
use crate::menu::Listener;
use crate::prelude::*;
use crate::window::ReaderWindow;
use crate::{library, reader};
use tauri::menu::MenuId;
use tauri_plugin_clipboard_manager::ClipboardExt;

#[derive(Debug, Display, EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum Item {
  AddBookToLibrary,
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
  fn execute(window: &Window, event: &MenuEvent) {
    let app = window.app_handle().clone();
    let label = window.label().to_owned();
    let menu_id = event.id().to_owned();
    spawn(async move {
      let Some(window_id) = reader::get_window_id_by_label(&app, &label).await else {
        return;
      };

      if let Some(item) = Self::from_menu_id(&menu_id, window_id) {
        debug!(menu_event = %item, reader_window = window_id);
        match item {
          Item::AddBookToLibrary => add_to_library(&app, window_id).await,
          Item::Close => close_reader_window(&app, &label),
          Item::CloseAll => close_all_reader_windows(&app).await,
          Item::CloseOthers => close_other_reader_windows(&app, window_id).await,
          Item::CopyBookPathToClipboard => copy_path_to_clipboard(&app, window_id).await,
          Item::OpenBookFolder => open_book_folder(&app, window_id).await,
        }
      };
    });
  }
}

pub fn build<M: Manager<Wry>>(app: &M, window_id: u16) -> Result<Menu<Wry>> {
  let menu = Menu::new(app)?;
  menu.append(&file_menu(app, window_id)?)?;

  Ok(menu)
}

fn file_menu<M: Manager<Wry>>(app: &M, window_id: u16) -> Result<Submenu<Wry>> {
  SubmenuBuilder::new(app, "File")
    .items(&[
      &menu_item!(app, Item::Close.to_menu_id(window_id), "Close")?,
      &menu_item!(app, Item::CloseAll.to_menu_id(window_id), "Close all")?,
      &menu_item!(app, Item::CloseOthers.to_menu_id(window_id), "Close others")?,
    ])
    .separator()
    .items(&[&menu_item!(
      app,
      Item::AddBookToLibrary.to_menu_id(window_id),
      "Add to library"
    )?])
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
    .build()
    .map_err(Into::into)
}

async fn add_to_library(app: &AppHandle, window_id: u16) {
  if let Some(path) = reader::get_book_path(app, window_id).await {
    let result: Result<()> = try {
      if let Some(model) = library::save(app, path).await? {
        // Disable the menu item after adding the book to the library.
        ReaderWindow::update_all_menus(app).await?;

        ActiveBook::from_model(app, &model)?
          .extract_cover(app)
          .await?;
      }
    };

    result.dialog(app);
  };
}

fn close_reader_window(app: &AppHandle, label: &str) {
  if let Some(window) = app.get_webview_window(label) {
    window.close().dialog(app);
  }
}

async fn close_all_reader_windows(app: &AppHandle) {
  reader::close_all(app).await.dialog(app);
}

async fn close_other_reader_windows(app: &AppHandle, window_id: u16) {
  reader::close_others(app, window_id)
    .await
    .dialog(app);
}

async fn copy_path_to_clipboard(app: &AppHandle, window_id: u16) {
  let path = reader::get_book_path(app, window_id)
    .await
    .and_then(|it| it.try_string().ok());

  if let Some(path) = path {
    app.clipboard().write_text(path).dialog(app);
  }
}

async fn open_book_folder(app: &AppHandle, window_id: u16) {
  let dir = reader::get_book_path(app, window_id)
    .await
    .and_then(|it| it.parent().map(ToOwned::to_owned));

  if matches!(dir, Some(ref it) if it.is_dir()) {
    open::that_detached(dir.unwrap()).dialog(app);
  }
}
