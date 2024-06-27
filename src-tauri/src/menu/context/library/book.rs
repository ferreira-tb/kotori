use crate::book::ActiveBook;
use crate::menu::prelude::*;
use crate::reader;
use crate::{library, menu_item_or_bail, prelude::*};
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

pub struct LibraryBookContextMenu {
  pub menu: Menu<Wry>,
  pub ctx: Mutex<Context>,
}

#[derive(Clone, Debug)]
pub struct Context {
  pub book_id: i32,
}

pub fn build<M: Manager<Wry>>(app: &M) -> Result<Menu<Wry>> {
  MenuBuilder::new(app)
    .items(&[
      &menu_item!(app, Item::OpenBook, "Open")?,
      &menu_item!(app, Item::RemoveBook, "Remove")?,
    ])
    .build()
    .map_err(Into::into)
}

pub async fn open_book(app: &AppHandle) {
  let state = app.state::<LibraryBookContextMenu>();
  let id = state.ctx.lock().unwrap().book_id;

  if let Ok(book) = ActiveBook::from_id(app, id).await {
    reader::open_book(app, book)
      .await
      .into_dialog(app);
  }
}

pub async fn remove_book(app: &AppHandle) {
  let state = app.state::<LibraryBookContextMenu>();
  let id = state.ctx.lock().unwrap().book_id;

  library::remove_with_dialog(app, id)
    .await
    .into_dialog(app);
}
