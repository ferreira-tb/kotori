use super::prelude::*;
use crate::book::ActiveBook;
use crate::prelude::*;

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

fn file_menu<M, R>(app: &M) -> Result<Submenu<R>>
where
  R: Runtime,
  M: Manager<R>,
{
  let mut menu = SubmenuBuilder::new(app, "File").items(&[
    &menu_item!(app, Id::OpenBook, "Open file", "Ctrl+O")?,
    &menu_item!(app, Id::AddToLibrary, "Add to library", "Ctrl+Shift+A")?,
  ]);

  // Not supported on Linux.
  // https://docs.rs/tauri/2.0.0-beta/tauri/menu/struct.SubmenuBuilder.html#method.quit
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
        Id::AddToLibrary => add_to_library_from_dialog(&app),
        Id::OpenBook => open_book_from_dialog(&app),
      }
    }
  }
}

fn add_to_library_from_dialog(app: &AppHandle) {
  let app = app.clone();
  async_runtime::spawn(async move {
    let kotori = app.state::<Kotori>();
    kotori.library.add_from_dialog().await.ok();
  });
}

fn open_book_from_dialog(app: &AppHandle) {
  let app = app.clone();
  async_runtime::spawn(async move {
    ActiveBook::open_book_from_dialog(&app).await.ok();
  });
}
