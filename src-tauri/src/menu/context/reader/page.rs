use crate::book::ActiveBook;
use crate::database::prelude::*;
use crate::menu::prelude::*;
use crate::{menu_item_or_bail, prelude::*, reader};

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
    async_runtime::spawn(async move {
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
  pub page: usize,
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
  let ctx = state.ctx.lock().await;

  reader::delete_page_with_dialog(app, ctx.window_id, ctx.page)
    .await
    .into_dialog(app);
}

async fn set_as_cover(app: &AppHandle) {
  let (book_id, page) = {
    let state = app.state::<ReaderPageContextMenu>();
    let ctx = state.ctx.lock().await;
    (ctx.book_id, ctx.page)
  };

  let Some(book_id) = book_id else {
    return;
  };

  let book = Book::get_by_id(app, book_id)
    .await
    .ok()
    .and_then(|model| ActiveBook::with_model(&model).ok());

  if let Some(book) = book {
    book
      .update_cover(app, page)
      .await
      .into_dialog(app);
  }
}
