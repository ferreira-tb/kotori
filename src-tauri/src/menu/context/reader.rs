pub mod page {
  use crate::book::ActiveBook;
  use crate::database::prelude::*;
  use crate::menu::prelude::*;
  use crate::prelude::*;

  #[derive(Display, EnumString)]
  enum Id {
    SetAsCover,
  }

  pub fn build<M, R>(app: &M, book_id: Option<i32>) -> Result<Menu<R>>
  where
    R: Runtime,
    M: Manager<R>,
  {
    let set_as_cover = MenuItemBuilder::new("Set as cover")
      .id(Id::SetAsCover)
      .enabled(book_id.is_some())
      .build(app)?;

    let menu = MenuBuilder::new(app)
      .items(&[&set_as_cover])
      .build()?;

    Ok(menu)
  }

  pub fn on_event<R>(app: &AppHandle, book_id: i32, page: usize) -> impl Fn(&Window<R>, MenuEvent)
  where
    R: Runtime,
  {
    let app = app.clone();
    move |_, event| {
      if let Ok(id) = Id::from_str(event.id.0.as_str()) {
        match id {
          Id::SetAsCover => set_as_cover(&app, book_id, page),
        }
      }
    }
  }

  fn set_as_cover(app: &AppHandle, book_id: i32, page: usize) {
    let app = app.clone();
    async_runtime::spawn(async move {
      let kotori = app.state::<Kotori>();
      let book = Book::find_by_id(book_id)
        .one(&kotori.db)
        .await
        .ok()
        .flatten()
        .and_then(|model| ActiveBook::with_model(&model).ok());

      if let Some(book) = book {
        book.update_cover(&app, page).await.ok();
      }
    });
  }
}
