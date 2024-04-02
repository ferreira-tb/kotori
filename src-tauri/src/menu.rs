use crate::book::ActiveBook;
use crate::prelude::*;
use std::str::FromStr;
use strum::{Display, EnumString};
use tauri::menu::{Menu, MenuEvent, MenuItemBuilder, Submenu, SubmenuBuilder};
use tauri::Window;

#[derive(Display, EnumString)]
enum Id {
  AddToLibrary,
  OpenBook,
}

pub fn build<M, R>(app: &mut M) -> Result<Menu<R>>
where
  R: Runtime,
  M: Manager<R>,
{
  let menu = Menu::new(app)?;
  menu.append(&file_menu(app)?)?;

  Ok(menu)
}

macro_rules! menu_item {
  ($app:expr, $id:ident, $text:literal) => {{
    MenuItemBuilder::with_id(Id::$id, $text).build($app)
  }};
  ($app:expr, $id:ident, $text:literal, $accelerator:literal) => {{
    MenuItemBuilder::with_id(Id::$id, $text)
      .accelerator($accelerator)
      .build($app)
  }};
}

fn file_menu<M, R>(app: &mut M) -> Result<Submenu<R>>
where
  R: Runtime,
  M: Manager<R>,
{
  let menu = SubmenuBuilder::new(app, "File")
    .item(&menu_item!(app, OpenBook, "Open file")?)
    .item(&menu_item!(app, AddToLibrary, "Add to library")?)
    .build()?;

  Ok(menu)
}

#[allow(clippy::needless_pass_by_value)]
pub fn event_handler<R>(app: AppHandle) -> impl Fn(&Window<R>, MenuEvent)
where
  R: Runtime,
{
  move |_, event| {
    if let Ok(id) = Id::from_str(event.id.0.as_str()) {
      match id {
        Id::AddToLibrary => {}
        Id::OpenBook => {
          let app = app.clone();
          async_runtime::spawn(async move {
            if let Some(book) = ActiveBook::from_dialog(&app).await.unwrap() {
              let kotori = app.state::<Kotori>();
              let mut reader = kotori.reader.lock().await;
              reader.open_book(book).await.unwrap();
            }
          });
        }
      }
    }
  }
}
