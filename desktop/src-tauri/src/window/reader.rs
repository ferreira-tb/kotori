use super::{ColorMode, WindowExt, WindowKind, WindowManager};
use crate::book::ActiveBook;
use crate::event::Event;
use crate::menu::{MenuExt, ReaderMenu, ReaderMenuItem};
use crate::prelude::*;
use crate::utils::glob;
use tauri::menu::{Menu, MenuEvent};
use tauri::{DragDropEvent, WebviewWindowBuilder, WindowEvent};

pub struct ReaderWindow {
  pub id: u16,
  pub book: ActiveBook,
}

impl ReaderWindow {
  pub fn open(app: &AppHandle, book: ActiveBook) -> Result<Self> {
    let window_id = get_available_id(app);
    let kind = WindowKind::Reader(window_id);
    let label = kind.label();
    let url = kind.url();
    let script = initialization_script(window_id);

    trace!(?kind, ?url);
    trace!(%script);

    let window = WebviewWindowBuilder::new(app, label, url)
      .initialization_script(&script)
      .data_directory(kind.data_dir(app)?)
      .title(book.title.to_string())
      .theme(ColorMode::get(app)?.into())
      .resizable(true)
      .maximizable(true)
      .minimizable(true)
      .visible(false)
      .build()?;

    window.on_window_event(on_window_event(app, window_id));

    let menu = ReaderMenu::build(app, window_id)?;
    window.set_menu(menu)?;
    window.on_menu_event(on_menu_event());

    // We should keep this hidden by default.
    // The user may toggle it visible, however.
    window.hide_menu()?;

    #[cfg(feature = "open-reader-devtools")]
    window.open_devtools();

    Ok(ReaderWindow { id: window_id, book })
  }

  pub async fn update_all_menus(app: &AppHandle) -> Result<()> {
    let windows = app.reader_windows();
    let windows = windows.read().await;

    for window in windows.values() {
      let menu = window.menu(app)?;

      let item = ReaderMenuItem::AddBookToLibrary.to_menu_id(window.id);
      let book_id = window.book.try_id().await.ok();
      menu.set_item_enabled(&item, book_id.is_none())?;

      let item = ReaderMenuItem::CloseOthers.to_menu_id(window.id);
      menu.set_item_enabled(&item, windows.len() > 1)?;
    }

    Ok(())
  }

  pub fn webview_window(&self, app: &AppHandle) -> Option<WebviewWindow> {
    get_reader_window(app, self.id)
  }

  fn menu(&self, app: &AppHandle) -> Result<Menu<Wry>> {
    self
      .webview_window(app)
      .and_then(|it| it.menu())
      .ok_or_else(|| err!(WindowMenuNotFound))
  }

  fn set_book(&mut self, app: &AppHandle, book: ActiveBook) -> Result<()> {
    if let Some(webview) = self.webview_window(app) {
      webview.set_title(book.title.as_str())?;
    };

    self.book = book;

    Event::ReaderBookChanged { window_id: self.id }.emit(app)
  }
}

fn get_available_id(app: &AppHandle) -> u16 {
  let mut id = 0;
  while get_reader_window(app, id).is_some() {
    id += 1;
  }

  id
}

fn get_reader_window(app: &AppHandle, id: u16) -> Option<WebviewWindow> {
  app.get_webview_window(&WindowKind::Reader(id).label())
}

fn initialization_script(id: u16) -> String {
  format!("window.KOTORI = {{ readerWindowId: {id} }}")
}

fn on_menu_event() -> impl Fn(&Window, MenuEvent) {
  use crate::menu::{Listener, ReaderMenuItem, ReaderPageContextMenuItem};

  move |window, event| {
    ReaderMenuItem::execute(window, &event);
    ReaderPageContextMenuItem::execute(window, &event);
  }
}

fn on_window_event(app: &AppHandle, window_id: u16) -> impl Fn(&WindowEvent) {
  let app = app.clone();
  move |event| match event {
    WindowEvent::CloseRequested { .. } => {
      trace!(close_requested = WindowKind::Reader(window_id).label());
      handle_close_requested_event(&app, window_id);
    }
    WindowEvent::DragDrop(DragDropEvent::Drop { paths, .. }) => {
      trace!(dropped = ?paths);
      handle_drop_event(&app, window_id, paths);
    }
    _ => {}
  }
}

/// When a reader window is closed, it should be removed from the list of windows.
/// After that, all remaining windows should have their menus updated to reflect this change.
/// The previous reader window should also be focused, or the main window if there are no more reader windows left.
fn handle_close_requested_event(app: &AppHandle, window_id: u16) {
  let app = app.clone();
  spawn(async move {
    let reader_arc = app.reader_windows();
    let mut windows = reader_arc.write().await;

    let previous_window_id = windows
      .get_index_of(&window_id)
      .and_then(|id| id.checked_sub(1))
      .unwrap_or(0);

    windows.shift_remove(&window_id);

    drop(windows);

    // This will read lock the windows.
    ReaderWindow::update_all_menus(&app)
      .await
      .into_err_log(&app);

    reader_arc
      .read()
      .await
      .get_index(previous_window_id)
      .and_then(|(_, window)| window.webview_window(&app))
      .or_else(|| Some(app.main_window()))
      .map(|webview| webview.set_foreground_focus())
      .transpose()
      .into_err_log(&app);
  });
}

/// Check for books among the dropped files and open the first one found.
/// If a book is opened successfully, update the menus for all reader windows.
fn handle_drop_event(app: &AppHandle, window_id: u16, paths: &[PathBuf]) {
  let globset = glob::book();
  let books = paths
    .iter()
    .filter(|it| globset.is_match(it))
    .collect_vec();

  if let Some(path) = books.first() {
    let app = app.clone();
    let path = (*path).clone();
    spawn(async move {
      let result: Result<()> = try {
        let windows = app.reader_windows();
        let mut windows = windows.write().await;

        if let Some(window) = windows.get_mut(&window_id) {
          let book = ActiveBook::new(&app, path)?;
          window.set_book(&app, book)?;
        }

        drop(windows);

        // This will read lock the windows.
        ReaderWindow::update_all_menus(&app).await?;
      };

      result.into_err_dialog(&app);
    });
  }
}