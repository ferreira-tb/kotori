use crate::menu::context::ContextMenuUpdate;
use crate::menu::prelude::*;
use crate::menu::Listener;
use crate::prelude::*;
use crate::reader;
use std::sync::Mutex;
use tauri_plugin_dialog::DialogExt;

#[derive(Debug, Display, EnumString)]
pub enum Item {
  #[strum(serialize = "kt-ctx-reader-page-delete-page")]
  DeletePage,
  #[strum(serialize = "kt-ctx-reader-page-export-page")]
  ExportPage,
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
        Item::ExportPage => export_page(&app).await,
        Item::SetAsCover => set_as_cover(&app).await,
      }
    });
  }
}

#[derive(Clone, Debug)]
pub struct Context {
  pub window_id: u16,
  pub book_id: Option<i32>,
  pub page_name: String,
}

pub struct ReaderPageContextMenu {
  pub menu: Menu<Wry>,
  pub ctx: Mutex<Context>,
}

impl ReaderPageContextMenu {
  fn new<M: Manager<Wry>>(app: &M, ctx: Context) -> Result<Self> {
    let menu = MenuBuilder::new(app)
      .items(&[
        &mi!(app, SetAsCover, "Set as cover")?,
        &mi!(app, ExportPage, "Export page")?,
        &PredefinedMenuItem::separator(app)?,
        &mi!(app, DeletePage, "Delete page")?,
      ])
      .build()?;

    let ctx = Mutex::new(ctx);
    Ok(Self { menu, ctx })
  }

  fn context(app: &AppHandle) -> Context {
    let state = app.state::<Self>();
    let ctx = state.ctx.lock().unwrap();
    ctx.clone()
  }

  pub fn popup(window: &Window, ctx: Context) -> Result<()> {
    popup_context_menu!(window, ReaderPageContextMenu, ctx)
  }
}

impl ContextMenuUpdate for ReaderPageContextMenu {
  type Context = Context;
}

async fn delete_page(app: &AppHandle) {
  let ctx = ReaderPageContextMenu::context(app);
  reader::delete_page_with_dialog(app, ctx.window_id, &ctx.page_name)
    .await
    .into_err_dialog(app);
}

async fn export_page(app: &AppHandle) {
  let ctx = ReaderPageContextMenu::context(app);
  let windows = app.reader_windows();
  let windows = windows.read().await;

  if let Some(window) = windows.get(&ctx.window_id) {
    let Ok(bytes) = window
      .book
      .get_page_as_bytes(&ctx.page_name)
      .await
    else {
      let err = Error::PageNotFound(ctx.page_name.clone());
      Err::<(), _>(err).into_err_dialog(app);
      return;
    };

    drop(windows);

    let file_name = Path::new(&ctx.page_name)
      .file_name()
      .and_then(|it| it.try_string().ok())
      .unwrap_or_else(|| String::from("page"));

    let app = app.clone();
    app
      .dialog()
      .file()
      .set_title("Export page")
      .set_file_name(file_name)
      .save_file(move |path| {
        if let Some(path) = path {
          #[cfg(feature = "tracing")]
          info!("exporting page to {:?}", path);

          std::fs::write(path, bytes)
            .map_err(Into::into)
            .into_err_dialog(&app);
        }
      });
  } else {
    #[cfg(feature = "tracing")]
    {
      use crate::window::WindowKind;

      let label = WindowKind::Reader(ctx.window_id).label();
      warn!("failed to export page, window not found: {label}");
    }
  }
}

async fn set_as_cover(app: &AppHandle) {
  let ctx = ReaderPageContextMenu::context(app);
  if let Some(book_id) = ctx.book_id {
    app
      .database_handle()
      .update_book_cover(book_id, &ctx.page_name)
      .await
      .into_err_dialog(app);
  };
}
