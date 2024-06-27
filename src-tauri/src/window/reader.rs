use super::WindowKind;
use crate::book::ActiveBook;
use crate::menu::reader::{build as build_menu, Item};
use crate::menu::MenuExt;
use crate::prelude::*;
use tauri::menu::MenuEvent;
use tauri::{WebviewWindowBuilder, WindowEvent};

pub struct ReaderWindow {
  pub id: u16,
  pub book: ActiveBook,
}

impl ReaderWindow {
  pub async fn open(app: &AppHandle, book: ActiveBook) -> Result<Self> {
    let window_id = get_available_id(app);
    let script = format!("window.KOTORI = {{ readerWindowId: {window_id} }}");
    trace!(%script);

    let kind = WindowKind::Reader(window_id);
    let webview = WebviewWindowBuilder::new(app, kind.label(), kind.url())
      .initialization_script(&script)
      .data_directory(kind.data_dir(app)?)
      .title(book.title.to_string())
      .resizable(true)
      .maximizable(true)
      .minimizable(true)
      .visible(false)
      .build()?;

    on_window_event(app, &webview, window_id);

    let menu = build_menu(app, window_id)?;

    let book_id = book.try_id(app).await.ok();
    menu.set_item_enabled(
      &Item::AddBookToLibrary.to_menu_id(window_id),
      book_id.is_none(),
    )?;

    webview.set_menu(menu)?;
    webview.on_menu_event(on_menu_event());

    // We should keep this hidden by default.
    // The user may toggle it visible, however.
    webview.hide_menu()?;

    Ok(ReaderWindow { id: window_id, book })
  }

  pub fn webview(&self, app: &AppHandle) -> Option<WebviewWindow> {
    get_reader_window(app, self.id)
  }

  pub fn set_menu_item_enabled(&self, app: &AppHandle, item: &Item, enabled: bool) -> Result<()> {
    let menu = self.webview(app).and_then(|it| it.menu());
    if let Some(menu) = menu {
      let id = item.to_menu_id(self.id);
      menu.set_item_enabled(&id, enabled)?;
    }

    Ok(())
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

fn on_menu_event() -> impl Fn(&Window, MenuEvent) {
  use crate::menu::{self, context, Listener};
  move |window, event| {
    menu::reader::Item::execute(window, &event);
    context::reader::page::Item::execute(window, &event);
  }
}

fn on_window_event(app: &AppHandle, webview: &WebviewWindow, window_id: u16) {
  let app = app.clone();
  webview.on_window_event(move |event| {
    if matches!(event, WindowEvent::CloseRequested { .. }) {
      let app = app.clone();
      spawn(async move {
        info!("close requested, {}", WindowKind::Reader(window_id).label());
        let reader_arc = app.reader_windows();
        let mut windows = reader_arc.write().await;

        let previous = windows
          .get_index_of(&window_id)
          .and_then(|id| id.checked_sub(1))
          .unwrap_or(0);

        windows.shift_remove(&window_id);

        // No need to hold a write lock from now on.
        drop(windows);

        reader_arc
          .read()
          .await
          .get_index(previous)
          .and_then(|(_, window)| window.webview(&app))
          .or_else(|| Some(app.main_window()))
          .map(|webview| webview.set_focus())
          .transpose()
          .into_log(&app);
      });
    }
  });
}
