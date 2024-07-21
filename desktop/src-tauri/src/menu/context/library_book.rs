use crate::book::ActiveBook;
use crate::menu::prelude::*;
use crate::menu::Listener;
use crate::prelude::*;
use crate::{library, menu_item_or_bail, popup_context_menu, reader};
use std::sync::Mutex;

#[derive(Debug, Display, EnumString)]
pub enum Item {
  #[strum(serialize = "kt-ctx-book-open-book")]
  OpenBook,
  #[strum(serialize = "kt-ctx-book-remove-book")]
  RemoveBook,
}

impl Listener for Item {
  fn execute(window: &Window, event: &MenuEvent) {
    let item = menu_item_or_bail!(event);
    let app = window.app_handle().clone();
    spawn(async move {
      match item {
        Item::OpenBook => open_book(&app).await,
        Item::RemoveBook => remove_book(&app).await,
      }
    });
  }
}

#[derive(Clone, Debug)]
pub struct Context {
  pub book_id: i32,
}

pub struct LibraryBookContextMenu {
  pub menu: Menu<Wry>,
  pub ctx: Mutex<Context>,
}

impl LibraryBookContextMenu {
  fn new<M: Manager<Wry>>(app: &M, ctx: Context) -> Result<Self> {
    let menu = MenuBuilder::new(app)
      .items(&[
        &menu_item!(app, Item::OpenBook, "Open")?,
        &menu_item!(app, Item::RemoveBook, "Remove")?,
      ])
      .build()?;

    let ctx = Mutex::new(ctx);
    Ok(Self { menu, ctx })
  }

  pub fn popup(window: &Window, ctx: Context) -> Result<()> {
    popup_context_menu!(window, LibraryBookContextMenu, ctx)
  }
}

async fn open_book(app: &AppHandle) {
  let state = app.state::<LibraryBookContextMenu>();
  let id = state.ctx.lock().unwrap().book_id;

  if let Ok(book) = ActiveBook::from_id(app, id).await {
    reader::open_book(app, book)
      .await
      .into_err_dialog(app);
  }
}

async fn remove_book(app: &AppHandle) {
  let state = app.state::<LibraryBookContextMenu>();
  let id = state.ctx.lock().unwrap().book_id;

  library::remove_with_dialog(app, id)
    .await
    .into_err_dialog(app);
}
