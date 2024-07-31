use crate::book::ActiveBook;
use crate::menu::prelude::*;
use crate::menu::Listener;
use crate::prelude::*;
use crate::{library, menu_item_or_bail, popup_context_menu, reader};
use std::sync::Mutex;

#[derive(Debug, Display, EnumString)]
pub enum Item {
  #[strum(serialize = "kt-ctx-library-book-open-book")]
  OpenBook,
  #[strum(serialize = "kt-ctx-library-book-open-book-folder")]
  OpenBookFolder,
  #[strum(serialize = "kt-ctx-library-book-remove-book")]
  RemoveBook,
}

impl Listener for Item {
  fn execute(window: &Window, event: &MenuEvent) {
    let item = menu_item_or_bail!(event);
    let app = window.app_handle().clone();
    spawn(async move {
      match item {
        Item::OpenBook => open_book(&app).await,
        Item::OpenBookFolder => open_book_folder(&app).await,
        Item::RemoveBook => remove_book(&app).await,
      }
    });
  }
}

#[derive(Clone, Debug)]
pub struct Context {
  pub book_id: i32,
}

impl Context {
  fn book_id(app: &AppHandle) -> i32 {
    app
      .state::<LibraryBookContextMenu>()
      .ctx
      .lock()
      .unwrap()
      .book_id
  }
}

pub struct LibraryBookContextMenu {
  pub menu: Menu<Wry>,
  pub ctx: Mutex<Context>,
}

impl LibraryBookContextMenu {
  fn new<M: Manager<Wry>>(app: &M, ctx: Context) -> Result<Self> {
    let menu = MenuBuilder::new(app)
      .items(&[
        &mi!(app, Item::OpenBook, "Open")?,
        &mi!(app, Item::RemoveBook, "Remove")?,
      ])
      .separator()
      .items(&[&mi!(app, Item::OpenBookFolder, "Open folder")?])
      .build()?;

    let ctx = Mutex::new(ctx);
    Ok(Self { menu, ctx })
  }

  pub fn popup(window: &Window, ctx: Context) -> Result<()> {
    popup_context_menu!(window, LibraryBookContextMenu, ctx)
  }
}

async fn open_book(app: &AppHandle) {
  let id = Context::book_id(app);
  if let Ok(book) = ActiveBook::from_id(app, id).await {
    reader::open_book(app, book)
      .await
      .into_err_dialog(app);
  }
}

async fn open_book_folder(app: &AppHandle) {
  let id = Context::book_id(app);
  let result: Result<()> = try {
    app
      .database_handle()
      .get_book_path(id)
      .await?
      .open_parent_detached()?
  };

  result.into_err_dialog(app);
}

async fn remove_book(app: &AppHandle) {
  let id = Context::book_id(app);
  library::remove_with_dialog(app, id)
    .await
    .into_err_dialog(app);
}
