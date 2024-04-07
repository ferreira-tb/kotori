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

pub fn build<M, R>(app: &M) -> Result<Menu<R>>
where
  R: Runtime,
  M: Manager<R>,
{
  let menu = Menu::new(app)?;
  menu.append(&file_menu(app)?)?;

  Ok(menu)
}

macro_rules! item {
  ($app:expr, $id:ident, $text:literal) => {{
    MenuItemBuilder::with_id(Id::$id, $text).build($app)
  }};
  ($app:expr, $id:ident, $text:literal, $accelerator:literal) => {{
    MenuItemBuilder::with_id(Id::$id, $text)
      .accelerator($accelerator)
      .build($app)
  }};
}

fn file_menu<M, R>(app: &M) -> Result<Submenu<R>>
where
  R: Runtime,
  M: Manager<R>,
{
  let mut menu = SubmenuBuilder::new(app, "File")
    .item(&item!(app, OpenBook, "Open file", "Ctrl+O")?)
    .item(&item!(app, AddToLibrary, "Add to library", "Ctrl+Shift+A")?);

  if !cfg!(target_os = "linux") {
    menu = menu.separator().quit();
  }

  menu.build().map_err(Into::into)
}

pub fn on_menu_event<R>(app: &AppHandle) -> impl Fn(&Window<R>, MenuEvent)
where
  R: Runtime,
{
  let app = app.clone();
  move |_, event| {
    if let Ok(id) = Id::from_str(event.id.0.as_str()) {
      match id {
        Id::AddToLibrary => {
          let app = app.clone();
          async_runtime::spawn(async move {
            let kotori = app.state::<Kotori>();
            kotori.library.add_from_dialog().await.ok();
          });
        }
        Id::OpenBook => {
          let app = app.clone();
          async_runtime::spawn(async move {
            ActiveBook::open_book_from_dialog(&app).await.ok();
          });
        }
      }
    }
  }
}
