use crate::book::ActiveBook;
use crate::database::prelude::*;
use crate::menu::prelude::*;
use crate::{menu_item_or_bail, prelude::*, reader};
use std::sync::Mutex;

#[derive(Debug, Display, EnumString)]
pub enum Item {
  #[strum(serialize = "kt-ctx-page-delete-page")]
  DeletePage,
  #[strum(serialize = "kt-ctx-page-set-as-cover")]
  SetAsCover,
}

impl Listener for Item {
  fn execute(window: &Window, event: &MenuEvent) {
    let item = menu_item_or_bail!(event);
    let app = window.app_handle().clone();
    spawn(async move {
      match item {
        Item::DeletePage => delete_page(&app).await,
        Item::SetAsCover => set_as_cover(&app).await,
      }
    });
  }
}

pub struct ReaderPageContextMenu {
  pub menu: Menu<Wry>,
  pub ctx: Mutex<Context>,
}

#[derive(Clone, Debug)]
pub struct Context {
  pub window_id: u16,
  pub book_id: Option<i32>,
  pub name: String,
}

pub fn build<M: Manager<Wry>>(app: &M) -> Result<Menu<Wry>> {
  MenuBuilder::new(app)
    .items(&[
      &menu_item!(app, Item::SetAsCover, "Set as cover")?,
      &PredefinedMenuItem::separator(app)?,
      &menu_item!(app, Item::DeletePage, "Delete")?,
    ])
    .build()
    .map_err(Into::into)
}

async fn delete_page(app: &AppHandle) {
  let state = app.state::<ReaderPageContextMenu>();
  let (window_id, name) = {
    let ctx = state.ctx.lock().unwrap();
    (ctx.window_id, ctx.name.clone())
  };

  reader::delete_page_with_dialog(app, window_id, &name)
    .await
    .dialog(app);
}

async fn set_as_cover(app: &AppHandle) {
  let (book_id, name) = {
    let state = app.state::<ReaderPageContextMenu>();
    let ctx = state.ctx.lock().unwrap();
    (ctx.book_id, ctx.name.clone())
  };

  if let Some(book_id) = book_id {
    let book = Book::get_by_id(app, book_id)
      .await
      .ok()
      .and_then(|model| ActiveBook::from_model(app, &model).ok());

    if let Some(book) = book {
      book
        .update_cover(app, &name)
        .await
        .dialog(app);
    }
  };
}
