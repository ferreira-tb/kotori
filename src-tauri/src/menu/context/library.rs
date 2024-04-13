pub mod book {
  use crate::book::{ActiveBook, Title};
  use crate::database::prelude::*;
  use crate::event::Event;
  use crate::menu::prelude::*;
  use crate::prelude::*;

  #[derive(Display, EnumString)]
  enum Id {
    OpenBook,
    RemoveBook,
  }

  pub fn build<M, R>(app: &M) -> Result<Menu<R>>
  where
    R: Runtime,
    M: Manager<R>,
  {
    let menu = MenuBuilder::new(app)
      .items(&[
        &menu_item!(app, Id::OpenBook, "Open")?,
        &menu_item!(app, Id::RemoveBook, "Remove")?,
      ])
      .build()?;

    Ok(menu)
  }

  pub fn on_event<R>(app: &AppHandle, book_id: i32) -> impl Fn(&Window<R>, MenuEvent)
  where
    R: Runtime,
  {
    let app = app.clone();
    move |_, event| {
      if let Ok(id) = Id::from_str(event.id.0.as_str()) {
        match id {
          Id::OpenBook => open_book(&app, book_id),
          Id::RemoveBook => remove_book(&app, book_id),
        }
      }
    }
  }

  pub fn open_book(app: &AppHandle, id: i32) {
    let app = app.clone();
    async_runtime::spawn(async move {
      if let Ok(book) = ActiveBook::from_id(&app, id).await {
        book
          .open(&app)
          .await
          .inspect_err(|err| error!("\"{err}\""))
          .ok();
      }
    });
  }

  pub fn remove_book(app: &AppHandle, id: i32) {
    let app = app.clone();
    async_runtime::spawn(async move {
      let kotori = app.kotori();
      let book = Book::find_by_id(id)
        .one(&kotori.db)
        .await
        .ok()
        .flatten()
        .map(|it| (it.id, Title::try_from(it.path.as_str())));

      if let Some((id, title)) = book {
        let Ok(title) = title.map(|it| it.to_string()) else {
          return;
        };

        Event::RemoveBookRequested { id, title }
          .emit(&app)
          .ok();
      }
    });
  }
}
