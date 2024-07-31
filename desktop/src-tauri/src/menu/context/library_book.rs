use crate::book::ActiveBook;
use crate::database::model::Book;
use crate::manager::ManagerExt;
use crate::menu::context::ContextMenuUpdate;
use crate::menu::prelude::*;
use crate::menu::Listener;
use crate::prelude::*;
use crate::{library, menu_item_or_bail, popup_context_menu, reader};
use std::ops::Deref;
use std::sync::Mutex;

#[derive(Debug, Display, EnumString)]
pub enum Item {
  #[strum(serialize = "kt-ctx-library-book-mark-as-read")]
  MarkAsRead,
  #[strum(serialize = "kt-ctx-library-book-mark-as-unread")]
  MarkAsUnread,
  #[strum(serialize = "kt-ctx-library-book-open-book")]
  OpenBook,
  #[strum(serialize = "kt-ctx-library-book-open-book-folder")]
  OpenBookFolder,
  #[strum(serialize = "kt-ctx-library-book-remove-book")]
  RemoveBook,
}

impl Item {
  pub fn to_menu_id(&self) -> MenuId {
    MenuId::new(self.to_string())
  }
}

impl Listener for Item {
  fn execute(window: &Window, event: &MenuEvent) {
    let item = menu_item_or_bail!(event);
    let app = window.app_handle().clone();
    spawn(async move {
      match item {
        Item::MarkAsRead => mark_as_read(&app, true).await,
        Item::MarkAsUnread => mark_as_read(&app, false).await,
        Item::OpenBook => open_book(&app).await,
        Item::OpenBookFolder => open_book_folder(&app).await,
        Item::RemoveBook => remove_book(&app).await,
      }
    });
  }
}

#[derive(Clone, Debug)]
pub struct Context {
  pub book: Book,
}

impl Context {
  pub async fn new<M: ManagerExt>(manager: &M, id: i32) -> Result<Self> {
    manager
      .database_handle()
      .get_book_by_id(id)
      .await
      .map(|book| Self { book })
  }

  fn book_id<M: Manager<Wry>>(manager: &M) -> i32 {
    manager
      .state::<LibraryBookContextMenu>()
      .ctx
      .lock()
      .unwrap()
      .book
      .id
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
      .items(&[&*MarkAs::new(app, &ctx)?])
      .separator()
      .items(&[&mi!(app, Item::OpenBookFolder, "Open folder")?])
      .build()?;

    Ok(Self { menu, ctx: Mutex::new(ctx) })
  }

  pub fn popup(window: &Window, ctx: Context) -> Result<()> {
    popup_context_menu!(window, LibraryBookContextMenu, ctx)
  }
}

impl ContextMenuUpdate for LibraryBookContextMenu {
  type Context = Context;

  fn update(&self, ctx: &Context) -> Result<()> {
    self
      .menu
      .set_item_checked(&Item::MarkAsRead.to_menu_id(), ctx.book.read)?;

    self
      .menu
      .set_item_checked(&Item::MarkAsUnread.to_menu_id(), !ctx.book.read)
  }
}

struct MarkAs(Submenu<Wry>);

impl MarkAs {
  fn new<M: Manager<Wry>>(app: &M, ctx: &Context) -> Result<Self> {
    let submenu = SubmenuBuilder::new(app, "Mark as")
      .items(&[
        &CheckMenuItemBuilder::with_id(Item::MarkAsRead, "Read")
          .checked(ctx.book.read)
          .build(app)?,
        &CheckMenuItemBuilder::with_id(Item::MarkAsUnread, "Unread")
          .checked(!ctx.book.read)
          .build(app)?,
      ])
      .build()?;

    Ok(Self(submenu))
  }
}

impl Deref for MarkAs {
  type Target = Submenu<Wry>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

async fn mark_as_read(app: &AppHandle, read: bool) {
  let id = Context::book_id(app);
  app
    .database_handle()
    .update_book_read(id, read)
    .await
    .into_err_dialog(app);
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
