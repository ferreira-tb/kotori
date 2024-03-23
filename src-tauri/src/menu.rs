use std::str::FromStr;

use crate::event::Event;
use crate::library::Library;
use crate::prelude::*;
use tauri::menu::{Menu, MenuBuilder, MenuEvent, MenuItemBuilder, SubmenuBuilder};
use tokio::task;

#[derive(strum::Display, strum::EnumString)]
enum Id {
  AddToLibrary,
  Library,
  OpenBook,
}

pub fn build<M, R>(app: &mut M) -> Result<Menu<R>>
where
  R: Runtime,
  M: Manager<R>,
{
  let menu = MenuBuilder::new(app).build()?;

  menu.append(
    &SubmenuBuilder::new(app, "File")
      .item(&MenuItemBuilder::with_id(Id::OpenBook, "Open file").build(app)?)
      .item(&MenuItemBuilder::with_id(Id::AddToLibrary, "Add to library").build(app)?)
      .separator()
      .quit()
      .build()?,
  )?;

  menu.append(
    &SubmenuBuilder::new(app, "Browse")
      .item(
        &MenuItemBuilder::with_id(Id::Library, "Library")
          .accelerator("F1")
          .build(app)?,
      )
      .build()?,
  )?;

  Ok(menu)
}

#[allow(clippy::needless_pass_by_value)]
pub fn event_handler(app: &AppHandle, event: MenuEvent) {
  if let Ok(id) = Id::from_str(event.id.0.as_str()) {
    match id {
      Id::AddToLibrary => {
        let app = app.clone();
        task::spawn(async move {
          Library::add_with_dialog(&app).await.unwrap();
        });
      }
      Id::Library => {
        app
          .emit(Event::NavigateToLibrary.as_str(), ())
          .ok();
      }
      Id::OpenBook => {
        let app = app.clone();
        task::spawn(async move {
          Library::open_with_dialog(&app).await.unwrap();
        });
      }
    }
  }
}
