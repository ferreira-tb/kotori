use crate::book::ActiveBook;
use crate::database::model::Book;
use crate::manager::ManagerExt;
use crate::menu::context::ContextMenuUpdate;
use crate::menu::prelude::*;
use crate::menu::Listener;
use crate::prelude::*;
use crate::{library, reader};
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
        Item::OpenBookFolder => open_book_folder(&app),
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
}

pub struct LibraryBookContextMenu {
  pub menu: Menu<Wry>,
  pub ctx: Mutex<Context>,
}

impl LibraryBookContextMenu {
  fn new<M: Manager<Wry>>(app: &M, ctx: Context) -> Result<Self> {
    let menu = MenuBuilder::new(app)
      .items(&[
        &mi!(app, OpenBook, "Open")?,
        &mi!(app, RemoveBook, "Remove")?,
      ])
      .separator()
      .items(&[&*MarkAsMenu::new(app, &ctx)?])
      .separator()
      .items(&[&mi!(app, OpenBookFolder, "Open folder")?])
      .build()?;

    Ok(Self { menu, ctx: Mutex::new(ctx) })
  }

  fn context(app: &AppHandle) -> Context {
    let state = app.state::<Self>();
    let ctx = state.ctx.lock().unwrap();
    ctx.clone()
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

struct MarkAsMenu(Submenu<Wry>);

impl MarkAsMenu {
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

impl_deref_menu!(MarkAsMenu);

async fn mark_as_read(app: &AppHandle, read: bool) {
  let id = LibraryBookContextMenu::context(app).book.id;
  app
    .database_handle()
    .update_book_read(id, read)
    .await
    .into_err_dialog(app);
}

async fn open_book(app: &AppHandle) {
  let ctx = LibraryBookContextMenu::context(app);
  if let Ok(book) = ActiveBook::from_model(app, &ctx.book) {
    reader::open_book(app, book)
      .await
      .into_err_dialog(app);
  }
}

fn open_book_folder(app: &AppHandle) {
  let path = LibraryBookContextMenu::context(app).book.path;
  Path::new(&path)
    .open_parent_detached()
    .into_err_dialog(app);
}

async fn remove_book(app: &AppHandle) {
  let id = LibraryBookContextMenu::context(app).book.id;
  library::remove_with_dialog(app, id)
    .await
    .into_err_dialog(app);
}
