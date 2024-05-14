use crate::book::ActiveBook;
use crate::menu::prelude::*;
use crate::{library, menu_item_or_bail, prelude::*};

#[derive(Debug, Display, EnumString)]
pub enum Item {
  #[strum(serialize = "kt-ctx-book-open-book")]
  OpenBook,
  #[strum(serialize = "kt-ctx-book-remove-book")]
  RemoveBook,
}

impl Listener for Item {
  type Context = Context;

  fn execute(app: &AppHandle, _: &Window, event: &MenuEvent, ctx: Self::Context) {
    let item = menu_item_or_bail!(event);
    match item {
      Item::OpenBook => open_book(app, ctx.book_id),
      Item::RemoveBook => remove_book(app, ctx.book_id),
    }
  }
}

#[derive(Clone)]
pub struct Context {
  pub book_id: i32,
}

pub fn build<M, R>(app: &M) -> Result<Menu<R>>
where
  R: Runtime,
  M: Manager<R>,
{
  MenuBuilder::new(app)
    .items(&[
      &menu_item!(app, Item::OpenBook, "Open")?,
      &menu_item!(app, Item::RemoveBook, "Remove")?,
    ])
    .build()
    .map_err(Into::into)
}

pub fn open_book(app: &AppHandle, id: i32) {
  let app = app.clone();
  async_runtime::spawn(async move {
    if let Ok(book) = ActiveBook::from_id(&app, id).await {
      book.open(&app).await.into_dialog(&app);
    }
  });
}

pub fn remove_book(app: &AppHandle, id: i32) {
  let app = app.clone();
  async_runtime::spawn(async move {
    library::remove_with_dialog(&app, id)
      .await
      .into_dialog(&app);
  });
}
