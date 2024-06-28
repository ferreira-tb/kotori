use crate::book::handle::{BookHandle, PageMap};
use crate::book::title::Title;
use crate::book::Cover;
use crate::database::BookExt;
use crate::event::Event;
use crate::{library, prelude::*};
use image::ImageFormat;
use kotori_entity::book;
use kotori_entity::prelude::Book;
use natord::compare_ignore_case;
use std::cmp::Ordering;
use std::sync::Arc;
use tokio::sync::OnceCell;

#[derive(Clone, Debug)]
pub struct ActiveBook {
  pub path: PathBuf,
  pub title: Title,

  id: OnceCell<i32>,
  handle: BookHandle,
  pages: OnceCell<Arc<PageMap>>,
}

impl ActiveBook {
  pub fn new(app: &AppHandle, path: impl AsRef<Path>) -> Result<Self> {
    let path = path.as_ref();
    let book = Self {
      path: path.to_owned(),
      title: Title::try_from(path)?,

      id: OnceCell::new(),
      handle: app.book_handle().clone(),
      pages: OnceCell::new(),
    };

    Ok(book)
  }

  pub async fn from_id(app: &AppHandle, id: i32) -> Result<Self> {
    Book::get_by_id(app, id)
      .await
      .and_then(|model| Self::from_model(app, &model))
  }

  pub fn from_model(app: &AppHandle, model: &book::Model) -> Result<Self> {
    let book = Self::new(app, &model.path)?;
    let _ = book.id.set(model.id);
    Ok(book)
  }

  pub async fn random(app: &AppHandle) -> Result<Option<Self>> {
    if let Some(book) = Book::get_random(app).await? {
      Self::from_model(app, &book).map(Some)
    } else {
      Ok(None)
    }
  }

  pub fn id(&self) -> Option<i32> {
    self.id.get().copied()
  }

  pub async fn try_id(&self, app: &AppHandle) -> Result<i32> {
    let id = self.id.get_or_try_init(|| async {
      Book::get_by_path(app, &self.path)
        .await
        .map(|model| model.id)
        .map_err(Into::into)
    });

    id.await.copied()
  }

  pub async fn pages(&self) -> Result<Arc<PageMap>> {
    let pages = self.pages.get_or_try_init(|| async {
      self
        .handle
        .get_pages(&self.path)
        .await
        .map_err(Into::into)
    });

    pages.await.map(Arc::clone)
  }

  async fn model(&self, app: &AppHandle) -> Result<book::Model> {
    let id = self.try_id(app).await?;
    Book::get_by_id(app, id).await
  }

  pub async fn has_page(&self, name: &str) -> Result<bool> {
    self
      .pages()
      .await
      .map(|it| it.values().any(|it| it == name))
  }

  async fn get_first_page(&self) -> Result<String> {
    self
      .pages()
      .await?
      .first()
      .map(|(_, name)| name.to_owned())
      .ok_or_else(|| err!(PageNotFound))
  }

  pub async fn get_page_as_bytes(&self, name: &str) -> Result<Vec<u8>> {
    self.handle.read_page(&self.path, name).await
  }

  pub async fn get_cover_name(&self, app: &AppHandle) -> Result<String> {
    let mut model = self.model(app).await?;
    if let Some(cover) = model.cover.take() {
      if self.has_page(&cover).await? {
        return Ok(cover);
      }
    };

    let id = self.try_id(app).await?;
    let name = self.get_first_page().await?;
    Book::update_cover(app, id, name.as_str()).await?;

    Ok(name)
  }

  pub async fn get_cover_as_bytes(&self, app: &AppHandle) -> Result<Vec<u8>> {
    let name = self.get_cover_name(app).await?;
    self.handle.read_page(&self.path, name).await
  }

  pub async fn extract_cover(&self, app: &AppHandle, path: PathBuf) -> Result<()> {
    let name = self.get_cover_name(app).await?;
    let page = self.get_page_as_bytes(&name).await?;
    let format = image::guess_format(&page)
      .inspect_err(|error| warn!(%error))
      .or_else(|_| ImageFormat::from_path(name))?;

    Cover::extract(&path, page, format).await?;

    let id = self.try_id(app).await?;
    let path = path.as_ref();
    Event::CoverExtracted { id, path }.emit(app)
  }

  /// Set the specified page as the book cover.
  pub async fn update_cover(&self, app: &AppHandle, name: &str) -> Result<()> {
    let id = self.try_id(app).await?;
    Book::update_cover(app, id, name).await?;

    if let Ok(cover) = app.path().cover(id) {
      self.extract_cover(app, cover).await?;
    }

    Ok(())
  }

  pub async fn delete_page(&mut self, app: &AppHandle, name: &str) -> Result<()> {
    self.handle.delete_page(&self.path, name).await?;

    // As the page has been removed, we need to reset the cell.
    self.pages.take();

    // Next steps are exclusive to books in the library.
    if let Ok(id) = self.try_id(app).await {
      if self.pages().await?.is_empty() {
        return library::remove(app, id).await;
      }

      // Reset the cover if it was the deleted page.
      let cover = self.get_cover_name(app).await?;
      if cover == name {
        Book::update_cover(app, id, None).await?;
        if let Ok(cover) = app.path().cover(id) {
          self.extract_cover(app, cover).await?;
        }
      }
    }

    Ok(())
  }
}

impl Drop for ActiveBook {
  fn drop(&mut self) {
    let path = self.path.clone();
    let handle = self.handle.clone();
    spawn(async move { handle.close(path).await });

    trace!(dropped = %self.path.display());
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
