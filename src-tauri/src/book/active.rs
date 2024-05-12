use super::cover::Cover;
use super::handle::Handle;
use super::title::Title;
use crate::database::prelude::*;
use crate::event::Event;
use crate::utils::collections::OrderedMap;
use crate::{library, utils};
use crate::{prelude::*, reader};
use image::ImageFormat;
use natord::compare_ignore_case;
use std::cmp::Ordering;
use tokio::fs;

#[derive(Clone)]
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

  pub async fn from_id(app: &AppHandle, id: i32) -> Result<Self> {
    Book::get_by_id(app, id)
      .await
      .and_then(|model| Self::with_model(&model))
  }

  pub fn with_model(model: &BookModel) -> Result<Self> {
    let book = Self::new(&model.path)?;
    book.id.set(model.id).ok();

    Ok(book)
  }

  pub fn id(&self) -> Option<i32> {
    self.id.get().copied()
  }

  pub async fn id_or_try_init(&self, app: &AppHandle) -> Result<i32> {
    let id = self.id.get_or_try_init(|| async {
      let model = Book::get_by_path(app, &self.path).await?;
      Ok::<i32, Error>(model.id)
    });

    id.await.copied()
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

  pub async fn reload_pages(&mut self) -> Result<&OrderedMap<usize, String>> {
    self.handle.take();
    self.pages.take();
    self.pages_or_try_init().await
  }

  async fn model(&self, app: &AppHandle) -> Result<BookModel> {
    let id = self.id_or_try_init(app).await?;
    Book::get_by_id(app, id).await
  }

  pub async fn open(self, app: &AppHandle) -> Result<()> {
    reader::open_book(app, self).await
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
    handle.get_page_by_name(name).await
  }

  pub async fn get_cover(&self, app: &AppHandle) -> Result<Cover> {
    let id = self.id_or_try_init(app).await?;
    let path = Cover::path(app, id)?;
    if fs::try_exists(&path).await? {
      return Ok(path.into());
    }

    Ok(Cover::NotExtracted)
  }

  pub async fn get_cover_name(&self, app: &AppHandle) -> Result<String> {
    let mut model = self.model(app).await?;
    if let Some(cover) = model.cover.take() {
      let handle = self.handle_or_try_init().await?;
      if handle.has_page(&cover).await {
        return Ok(cover);
      }
    };

    let id = self.id_or_try_init(app).await?;
    let name = self.get_first_page_name().await?;
    Book::update_cover(app, id, name).await?;

    Ok(name.to_owned())
  }

  pub async fn get_cover_as_bytes(&self, app: &AppHandle) -> Result<Vec<u8>> {
    let name = self.get_cover_name(app).await?;
    let handle = self.handle_or_try_init().await?;
    handle.get_page_by_name(&name).await
  }

  pub fn extract_cover(self, app: &AppHandle, path: PathBuf) {
    let app = app.clone();
    async_runtime::spawn(async move {
      let name = self.get_cover_name(&app).await?;
      let handle = self.handle_or_try_init().await?;
      let page = handle.get_page_by_name(&name).await?;

      let parent = utils::path::parent(&path)?;
      fs::create_dir_all(parent).await?;

      let format = ImageFormat::from_path(name)?;
      Cover::resize(page, format, &path).await?;

      let id = self.id_or_try_init(&app).await?;
      let path = path.as_ref();
      Event::CoverExtracted { id, path }.emit(&app)?;

      Ok::<(), Error>(())
    });
  }

  /// Set the specified page as the book cover.
  pub async fn update_cover(self, app: &AppHandle, page: usize) -> Result<()> {
    let id = self.id_or_try_init(app).await?;
    let name = self.get_page_name(page).await?;
    Book::update_cover(app, id, name).await?;

    if let Ok(cover) = Cover::path(app, id) {
      self.extract_cover(app, cover);
    }

    Ok(())
  }

  pub async fn delete_page(mut self, app: &AppHandle, page: usize) -> Result<()> {
    let handle = self.handle_or_try_init().await?;
    let mut pages = handle.pages().await;
    let name = pages
      .swap_remove(&page)
      .ok_or_else(|| err!(PageNotFound))?;

    handle
      .delete_page_by_name(&self.path, &name)
      .await?;

    // Next steps are exclusive to books in the library.
    if let Ok(id) = self.id_or_try_init(app).await {
      info!("page {page} deleted from book {id}");

      if pages.is_empty() {
        info!("book {id} is empty, removing from library");
        return library::remove(app, id).await;
      }

      drop(pages);

      // Reset the cover if it was the deleted page.
      let cover = self.get_cover_name(app).await?;
      if cover == name {
        info!("book {id} had its cover deleted, resetting");
        Book::update_cover(app, id, None).await?;

        if let Ok(cover) = Cover::path(app, id) {
          self.reload_pages().await?;
          self.extract_cover(app, cover);
        }
      }
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
