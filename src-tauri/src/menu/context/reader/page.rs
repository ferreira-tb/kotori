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
  type Context = Context;

  fn execute(app: &AppHandle, _: &Window, event: &MenuEvent, ctx: Self::Context) {
    let item = menu_item_or_bail!(event);
    match item {
      Item::DeletePage => delete_page(app, ctx.window_id, ctx.page),
      Item::SetAsCover => set_as_cover(app, ctx.book_id, ctx.page),
    }
  }
}

#[derive(Clone)]
pub struct Context {
  pub window_id: u16,
  pub book_id: Option<i32>,
  pub page: usize,
}

pub fn build<M, R>(app: &M, book_id: Option<i32>) -> Result<Menu<R>>
where
  R: Runtime,
  M: Manager<R>,
{
  let cover = MenuItemBuilder::new("Set as cover")
    .id(Item::SetAsCover)
    .enabled(book_id.is_some())
    .build(app)?;

  MenuBuilder::new(app)
    .items(&[
      &cover,
      &PredefinedMenuItem::separator(app)?,
      &menu_item!(app, Item::DeletePage, "Delete")?,
    ])
    .build()
    .map_err(Into::into)
}

fn delete_page(app: &AppHandle, window_id: u16, page: usize) {
  let app = app.clone();
  async_runtime::spawn(async move {
    reader::delete_page_with_dialog(&app, window_id, page)
      .await
      .into_dialog(&app)
      .await;
  });
}

fn set_as_cover(app: &AppHandle, book_id: Option<i32>, page: usize) {
  let Some(book_id) = book_id else {
    return;
  };

  let app = app.clone();
  async_runtime::spawn(async move {
    let book = Book::get_by_id(&app, book_id)
      .await
      .ok()
      .and_then(|model| ActiveBook::with_model(&model).ok());

    if let Some(book) = book {
      book
        .update_cover(&app, page)
        .await
        .into_dialog(&app)
        .await;
    }
  });
}
