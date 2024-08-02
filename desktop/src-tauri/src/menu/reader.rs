use crate::menu::prelude::*;
use crate::menu::Listener;
use crate::prelude::*;
use crate::{library, reader};
use tauri::menu::MenuId;

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
        #[cfg(feature = "tracing")]
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

/// Build a menu item for the reader window.
macro_rules! mi {
  ($app:expr, $id:ident, $window_id:expr, $text:expr) => {{
    tauri::menu::MenuItemBuilder::new($text)
      .id(Item::$id.to_menu_id($window_id))
      .build($app)
  }};
}

pub struct ReaderMenu;

impl ReaderMenu {
  pub fn build<M: Manager<Wry>>(app: &M, window_id: u16) -> Result<Menu<Wry>> {
    let menu = Menu::new(app)?;
    menu.append(&*FileMenu::new(app, window_id)?)?;

    Ok(menu)
  }

  /// Update the menu items.
  ///
  /// **IMPORTANT**: This will **READ LOCK** the reader windows!
  ///
  /// *TODO: Can we avoid this lock?*
  pub async fn update(app: &AppHandle) -> Result<()> {
    let windows = app.reader_windows();
    let windows = windows.read().await;

    for window in windows.values() {
      let menu = window.menu(app)?;

      let item = Item::AddBookToLibrary.to_menu_id(window.id);
      let book_id = window.book.try_id().await.ok();
      menu.set_item_enabled(&item, book_id.is_none())?;

      let item = Item::CloseOthers.to_menu_id(window.id);
      menu.set_item_enabled(&item, windows.len() > 1)?;
    }

    Ok(())
  }

  /// Spawn a task to update the menu items.
  ///
  /// **IMPORTANT**: The task will **READ LOCK** the reader windows!
  pub fn spawn_update(app: &AppHandle) {
    let app = app.clone();
    spawn(async move {
      Self::update(&app).await.into_err_dialog(&app);
    });
  }
}

struct FileMenu(Submenu<Wry>);

impl FileMenu {
  fn new<M: Manager<Wry>>(app: &M, window_id: u16) -> Result<Self> {
    SubmenuBuilder::new(app, "File")
      .items(&[
        &mi!(app, Close, window_id, "Close")?,
        &mi!(app, CloseAll, window_id, "Close all")?,
        &mi!(app, CloseOthers, window_id, "Close others")?,
      ])
      .separator()
      .items(&[&mi!(app, AddBookToLibrary, window_id, "Add to library")?])
      .separator()
      .items(&[
        &mi!(app, CopyBookPathToClipboard, window_id, "Copy book path")?,
        &mi!(app, OpenBookFolder, window_id, "Open book folder")?,
      ])
      .build()
      .map(Self)
      .map_err(Into::into)
  }
}

impl_deref_menu!(FileMenu);

async fn add_to_library(app: &AppHandle, window_id: u16) {
  if let Some(path) = reader::get_book_path(app, window_id).await {
    // We must disable this menu item after the book is saved.
    library::save(app, &path)
      .await
      .inspect(|_| ReaderMenu::spawn_update(app))
      .into_err_dialog(app);
  };
}

fn close_reader_window(app: &AppHandle, label: &str) {
  if let Some(window) = app.get_webview_window(label) {
    window
      .close()
      .map_err(Into::into)
      .into_err_dialog(app);
  }
}

async fn close_all_reader_windows(app: &AppHandle) {
  reader::close_all(app).await.into_err_dialog(app);
}

async fn close_other_reader_windows(app: &AppHandle, window_id: u16) {
  reader::close_others(app, window_id)
    .await
    .into_err_dialog(app);
}

async fn copy_path_to_clipboard(app: &AppHandle, window_id: u16) {
  use tauri_plugin_clipboard_manager::ClipboardExt;

  let path = reader::get_book_path(app, window_id)
    .await
    .and_then(|it| it.try_string().ok());

  if let Some(path) = path {
    app
      .clipboard()
      .write_text(path)
      .map_err(Into::into)
      .into_err_dialog(app);
  }
}

async fn open_book_folder(app: &AppHandle, window_id: u16) {
  if let Some(path) = reader::get_book_path(app, window_id).await {
    path.open_parent_detached().into_err_dialog(app);
  }
}
