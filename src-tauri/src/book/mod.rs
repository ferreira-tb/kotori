mod cover;
mod handle;
mod json;
mod title;

pub use cover::Cover;
pub use json::{IntoJson, LibraryBook, ReaderBook};
pub use title::Title;

use crate::database::prelude::*;
use crate::event::Event;
use crate::prelude::*;
use crate::utils::OrderedMap;
use handle::Handle;
use image::ImageFormat;
use natord::compare_ignore_case;
use std::cmp::Ordering;
use tauri_plugin_dialog::{DialogExt, FileDialogBuilder};
use tokio::sync::OnceCell;

pub struct ActiveBook {
  pub path: PathBuf,
  pub title: Title,

  id: OnceCell<i32>,
  handle: OnceCell<Handle>,
  pages: OnceCell<OrderedMap<usize, String>>,
}

impl ActiveBook {
  pub fn new(path: impl AsRef<Path>) -> Result<Self> {
    let path = path.as_ref();
    let book = Self {
      path: path.to_owned(),
      title: Title::try_from(path)?,

      id: OnceCell::new(),
      handle: OnceCell::new(),
      pages: OnceCell::new(),
    };

    Ok(book)
  }

  pub fn with_model(model: &BookModel) -> Result<Self> {
    let book = Self::new(&model.path)?;
    book.id.set(model.id).ok();

    Ok(book)
  }

  pub async fn from_id(app: &AppHandle, id: i32) -> Result<Self> {
    let kotori = app.state::<Kotori>();
    Book::find_by_id(id)
      .one(&kotori.db)
      .await?
      .ok_or_else(|| err!(BookNotFound))
      .and_then(|model| Self::with_model(&model))
  }

  pub fn id(&self) -> Option<i32> {
    self.id.get().copied()
  }

  pub async fn id_or_try_init(&self, app: &AppHandle) -> Option<i32> {
    let id = self.id.get_or_try_init(|| async {
      let model = Book::find_by_path(app, &self.path).await?;
      Ok::<i32, Error>(model.id)
    });

    id.await.ok().copied()
  }

  async fn handle_or_try_init(&self) -> Result<&Handle> {
    let handle = self.handle.get_or_try_init(|| async {
      let handle = Handle::new(&self.path).await?;
      Ok::<Handle, Error>(handle)
    });

    handle.await
  }

  pub async fn pages_or_try_init(&self) -> Result<&OrderedMap<usize, String>> {
    let pages = self.pages.get_or_try_init(|| async {
      let handle = self.handle_or_try_init().await?;
      let pages = handle.pages().await;
      Ok::<OrderedMap<usize, String>, Error>(pages)
    });

    pages.await
  }

  async fn model(&self, app: &AppHandle) -> Result<BookModel> {
    if let Some(id) = self.id() {
      let kotori = app.state::<Kotori>();
      return Book::find_by_id(id)
        .one(&kotori.db)
        .await?
        .ok_or_else(|| err!(BookNotFound));
    }

    let model = Book::find_by_path(app, &self.path).await?;
    self.id.set(model.id).ok();
    Ok(model)
  }

  pub async fn open(self, app: &AppHandle) -> Result<()> {
    let kotori = app.state::<Kotori>();
    let reader = kotori.reader.read().await;
    reader.open_book(self).await
  }

  pub async fn open_from_dialog(app: &AppHandle) -> Result<()> {
    let books = Self::show_dialog(app).await?;

    if !books.is_empty() {
      let kotori = app.state::<Kotori>();
      let reader = kotori.reader.read().await;
      return reader.open_many(books).await.map_err(Into::into);
    }

    Ok(())
  }

  async fn show_dialog(app: &AppHandle) -> Result<Vec<Self>> {
    let (tx, rx) = oneshot::channel();
    let dialog = app.dialog().clone();

    FileDialogBuilder::new(dialog)
      .add_filter("Book", &["cbr", "cbz"])
      .pick_files(move |response| {
        tx.send(response).ok();
      });

    if let Some(response) = rx.await? {
      return response
        .into_iter()
        .map(|it| Self::new(it.path))
        .collect();
    }

    Ok(Vec::new())
  }

  async fn get_page_name(&self, page: usize) -> Result<&str> {
    let name = self
      .pages_or_try_init()
      .await?
      .get(&page)
      .ok_or_else(|| err!(PageNotFound))?;

    Ok(name)
  }

  async fn get_first_page_name(&self) -> Result<&str> {
    let name = self
      .pages_or_try_init()
      .await?
      .first()
      .map(|(_, name)| name)
      .ok_or_else(|| err!(PageNotFound))?;

    Ok(name)
  }

  pub async fn get_page_as_bytes(&self, page: usize) -> Result<Vec<u8>> {
    let name = self.get_page_name(page).await?;
    let handle = self.handle_or_try_init().await?;
    handle.by_name(name).await
  }

  pub async fn get_cover(&self, app: &AppHandle) -> Result<Cover> {
    let id = self
      .id_or_try_init(app)
      .await
      .ok_or_else(|| err!(BookNotFound))?;

    let path = Cover::path(app, id)?;
    if path.try_exists()? {
      return Ok(path.into());
    }

    Ok(Cover::NotExtracted)
  }

  async fn get_cover_name(&self, app: &AppHandle) -> Result<String> {
    let model = self.model(app).await?;
    if let Some(cover) = model.cover {
      return Ok(cover);
    };

    let name = self.get_first_page_name().await?;
    let mut model: BookActiveModel = model.into();
    model.cover = Set(Some(name.to_owned()));

    let kotori = app.state::<Kotori>();
    model.update(&kotori.db).await?;

    Ok(name.to_owned())
  }

  pub async fn get_cover_as_bytes(&self, app: &AppHandle) -> Result<Vec<u8>> {
    let name = self.get_cover_name(app).await?;
    let handle = self.handle_or_try_init().await?;
    handle.by_name(&name).await
  }

  pub fn extract_cover(self, app: &AppHandle, path: PathBuf) {
    let app = app.clone();
    async_runtime::spawn(async move {
      let name = self.get_cover_name(&app).await?;
      let handle = self.handle_or_try_init().await?;
      let page = handle.by_name(&name).await?;

      if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
      }

      let format = ImageFormat::from_path(name)?;
      Cover::resize(page, format, &path).await?;

      if let Some(id) = self.id_or_try_init(&app).await {
        let event = Event::CoverExtracted { id, path };
        event.emit(&app)?;
      }

      Ok::<(), Error>(())
    });
  }

  pub async fn update_cover(self, app: &AppHandle, page: usize) -> Result<()> {
    let name = self.get_page_name(page).await?;
    let model = self.model(app).await?;

    let mut model: BookActiveModel = model.into();
    model.cover = Set(Some(name.to_owned()));

    let kotori = app.state::<Kotori>();
    let model = model.update(&kotori.db).await?;

    if let Ok(cover) = Cover::path(app, model.id) {
      self.extract_cover(app, cover);
    }

    Ok(())
  }
}

impl PartialEq for ActiveBook {
  fn eq(&self, other: &Self) -> bool {
    self.path == other.path
  }
}

impl Eq for ActiveBook {}

impl PartialOrd for ActiveBook {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for ActiveBook {
  fn cmp(&self, other: &Self) -> Ordering {
    compare_ignore_case(&self.title.0, &other.title.0)
  }
}

impl TryFrom<&Path> for ActiveBook {
  type Error = crate::error::Error;

  fn try_from(path: &Path) -> Result<Self> {
    Self::new(path)
  }
}

impl TryFrom<BookModel> for ActiveBook {
  type Error = crate::error::Error;

  fn try_from(model: BookModel) -> Result<Self> {
    Self::with_model(&model)
  }
}
