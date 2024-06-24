use super::WindowKind;
use crate::book::ActiveBook;
use crate::menu::reader::{build as build_menu, Item};
use crate::menu::MenuExt;
use crate::{prelude::*, reader};
use tauri::menu::MenuEvent;
use tauri::{WebviewWindowBuilder, WindowEvent};

pub struct ReaderWindow {
  pub id: u16,
  pub book: ActiveBook,
  pub webview: WebviewWindow,
}

impl ReaderWindow {
  fn new(id: u16, book: ActiveBook, webview: WebviewWindow) -> Self {
    Self { id, book, webview }
  }

  pub async fn open(app: &AppHandle, book: ActiveBook) -> Result<(u16, Self)> {
    let window_id = get_available_id(app);
    let script = format!("window.KOTORI = {{ readerWindowId: {window_id} }}");
    trace!(%script);

    let kind = WindowKind::Reader(window_id);
    let window = WebviewWindowBuilder::new(app, kind.label(), kind.url())
      .initialization_script(&script)
      .data_directory(kind.data_dir(app)?)
      .title(book.title.to_string())
      .resizable(true)
      .maximizable(true)
      .minimizable(true)
      .maximized(true)
      .visible(false)
      .build()
      .map(|webview| Self::new(window_id, book, webview))?;

    on_window_event(app, &window.webview, window_id);

    let menu = build_menu(app, window_id)?;

    let book_id = window.book.try_id(app).await.ok();
    menu.set_item_enabled(
      &Item::AddBookToLibrary.to_menu_id(window_id),
      book_id.is_none(),
    )?;

    window.webview.set_menu(menu)?;
    window.webview.on_menu_event(on_menu_event());

    // We should keep this hidden by default.
    // The user may toggle it visible, however.
    window.webview.hide_menu()?;

    Ok((window_id, window))
  }

  pub fn set_menu_item_enabled(&self, item: &Item, enabled: bool) -> Result<()> {
    if let Some(menu) = self.webview.menu() {
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
  app.get_webview_window(&format!("reader-{id}"))
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
      info!("close requested, window_id: {window_id}");
      reader::remove_window(&app, window_id);
    }
  });
}
