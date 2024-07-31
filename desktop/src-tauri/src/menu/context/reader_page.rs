use crate::menu::context::ContextMenuUpdate;
use crate::menu::prelude::*;
use crate::menu::Listener;
use crate::prelude::*;
use crate::{menu_item_or_bail, popup_context_menu, reader};
use std::sync::Mutex;

#[derive(Debug, Display, EnumString)]
pub enum Item {
  #[strum(serialize = "kt-ctx-reader-page-delete-page")]
  DeletePage,
  #[strum(serialize = "kt-ctx-reader-page-set-as-cover")]
  SetAsCover,
}

impl Listener for Item {
  fn execute(window: &Window, event: &MenuEvent) {
    let item = menu_item_or_bail!(event);
    let app = window.app_handle().clone();
    spawn(async move {
      match item {
        Item::DeletePage => delete_page(&app).await,
        Item::SetAsCover => set_as_cover(&app).await,
      }
    });
  }
}

#[derive(Clone, Debug)]
pub struct Context {
  pub window_id: u16,
  pub book_id: Option<i32>,
  pub name: String,
}

pub struct ReaderPageContextMenu {
  pub menu: Menu<Wry>,
  pub ctx: Mutex<Context>,
}

impl ReaderPageContextMenu {
  fn new<M: Manager<Wry>>(app: &M, ctx: Context) -> Result<Self> {
    let menu = MenuBuilder::new(app)
      .items(&[
        &mi!(app, Item::SetAsCover, "Set as cover")?,
        &PredefinedMenuItem::separator(app)?,
        &mi!(app, Item::DeletePage, "Delete page")?,
      ])
      .build()?;

    let ctx = Mutex::new(ctx);
    Ok(Self { menu, ctx })
  }

  pub fn popup(window: &Window, ctx: Context) -> Result<()> {
    popup_context_menu!(window, ReaderPageContextMenu, ctx)
  }
}

impl ContextMenuUpdate for ReaderPageContextMenu {
  type Context = Context;
}

async fn delete_page(app: &AppHandle) {
  let state = app.state::<ReaderPageContextMenu>();
  let (window_id, page_name) = {
    let ctx = state.ctx.lock().unwrap();
    (ctx.window_id, ctx.name.clone())
  };

  reader::delete_page_with_dialog(app, window_id, &page_name)
    .await
    .into_err_dialog(app);
}

async fn set_as_cover(app: &AppHandle) {
  let (book_id, page_name) = {
    let state = app.state::<ReaderPageContextMenu>();
    let ctx = state.ctx.lock().unwrap();
    (ctx.book_id, ctx.name.clone())
  };

  if let Some(book_id) = book_id {
    app
      .database_handle()
      .update_book_cover(book_id, &page_name)
      .await
      .into_err_dialog(app);
  };
}
