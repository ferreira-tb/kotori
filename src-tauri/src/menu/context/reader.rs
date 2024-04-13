pub mod page {
  use crate::book::ActiveBook;
  use crate::database::prelude::*;
  use crate::event::Event;
  use crate::menu::prelude::*;
  use crate::prelude::*;

  #[derive(Display, EnumString)]
  enum Id {
    DeletePage,
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
      .items(&[
        &set_as_cover,
        &PredefinedMenuItem::separator(app)?,
        &menu_item!(app, Id::DeletePage, "Delete")?,
      ])
      .build()?;

    Ok(menu)
  }

  pub fn on_event<R: Runtime>(
    app: &AppHandle,
    window_id: u16,
    book_id: Option<i32>,
    page: usize,
  ) -> impl Fn(&Window<R>, MenuEvent) {
    let app = app.clone();
    move |_, event| {
      let Ok(id) = Id::from_str(event.id.0.as_str()) else {
        return;
      };

      match id {
        Id::DeletePage => delete_page(&app, window_id, page),
        Id::SetAsCover => {
          if let Some(book_id) = book_id {
            set_as_cover(&app, book_id, page);
          }
        }
      }
    }
  }

  fn delete_page(app: &AppHandle, window_id: u16, _page: usize) {
    Event::DeletePageRequested { window_id }
      .emit(app)
      .ok();
  }

  fn set_as_cover(app: &AppHandle, book_id: i32, page: usize) {
    let app = app.clone();
    async_runtime::spawn(async move {
      let kotori = app.kotori();
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
