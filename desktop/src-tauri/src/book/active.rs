use crate::book::cover::Cover;
use crate::book::handle::PageMap;
use crate::book::title::Title;
use crate::database::model::Book;
use crate::event::Event;
use crate::library;
use crate::prelude::*;
use image::ImageFormat;
use natord::compare_ignore_case;
use std::cmp::Ordering;
use std::fmt;
use std::sync::Arc;
use tokio::sync::OnceCell;

#[derive(Clone)]
pub struct ActiveBook {
  pub path: PathBuf,
  pub title: Title,

  app: AppHandle,
  id: OnceCell<i32>,
  pages: OnceCell<Arc<PageMap>>,
}

impl ActiveBook {
  pub fn new(app: &AppHandle, path: impl AsRef<Path>) -> Result<Self> {
    let path = path.as_ref();
    let book = Self {
      path: path.to_owned(),
      title: Title::try_from(path)?,

      app: app.clone(),
      id: OnceCell::new(),
      pages: OnceCell::new(),
    };

    Ok(book)
  }

  pub async fn from_id(app: &AppHandle, id: i32) -> Result<Self> {
    app
      .database_handle()
      .get_book_by_id(id)
      .await
      .and_then(|book| Self::from_model(app, &book))
  }

  pub fn from_model(app: &AppHandle, book: &Book) -> Result<Self> {
    let active = Self::new(app, &book.path)?;
    let _ = active.id.set(book.id);
    Ok(active)
  }

  pub async fn random(app: &AppHandle) -> Result<Option<Self>> {
    let random = app.database_handle().random_book().await?;
    if let Some(book) = random {
      Self::from_model(app, &book).map(Some)
    } else {
      Ok(None)
    }
  }

  pub fn id(&self) -> Option<i32> {
    self.id.get().copied()
  }

  pub async fn try_id(&self) -> Result<i32> {
    let id = self.id.get_or_try_init(|| async {
      self
        .app
        .database_handle()
        .get_book_by_path(&self.path)
        .await
        .map(|model| model.id)
        .map_err(Into::into)
    });

    id.await.copied()
  }

  pub async fn pages(&self) -> Result<Arc<PageMap>> {
    let pages = self.pages.get_or_try_init(|| async {
      self
        .app
        .book_handle()
        .get_pages(&self.path)
        .await
        .map_err(Into::into)
    });

    pages.await.map(Arc::clone)
  }

  pub async fn has_page(&self, name: &str) -> Result<bool> {
    self
      .pages()
      .await
      .map(|it| it.values().any(|it| it == name))
  }

  pub async fn get_page_as_bytes(&self, name: &str) -> Result<Vec<u8>> {
    self
      .app
      .book_handle()
      .read_page(&self.path, name)
      .await
  }

  /// Get cover name if the book is in the library.
  pub async fn get_cover_name(&self) -> Result<String> {
    let id = self.try_id().await?;
    let cover = self
      .app
      .database_handle()
      .get_book_cover(id)
      .await?;

    // The cover saved in the database may have been deleted from the book file.
    if self.has_page(&cover).await? {
      return Ok(cover);
    }

    self
      .app
      .book_handle()
      .get_first_page_name(&self.path)
      .await
  }

  pub async fn get_cover_as_bytes(&self) -> Result<Vec<u8>> {
    let name = self.get_cover_name().await?;
    self
      .app
      .book_handle()
      .read_page(&self.path, &name)
      .await
  }

  pub async fn extract_cover(&self) -> Result<()> {
    let name = self.get_cover_name().await?;
    let bytes = self.get_page_as_bytes(&name).await?;

    let format = match image::guess_format(&bytes) {
      Ok(format) => format,
      #[cfg_attr(not(feature = "tracing"), allow(unused_variables))]
      Err(error) => {
        #[cfg(feature = "tracing")]
        warn!("failed to guess image format: {error}");

        ImageFormat::from_path(name)?
      }
    };

    let id = self.try_id().await?;
    let path = self.app.path().cover(id)?;
    Cover::extract(&path, bytes, format).await?;

    let path = path.as_ref();
    Event::CoverExtracted { id, path }.emit(&self.app)
  }

  pub async fn delete_page(&mut self, name: &str) -> Result<()> {
    // `ActiveBook::get_cover_name` will always fail if the book isn't in the library.
    let is_cover = match self.get_cover_name().await {
      Ok(cover) => cover == name,
      Err(_) => self.id().is_some(),
    };

    self
      .app
      .book_handle()
      .delete_page(&self.path, name)
      .await?;

    // As the page has been removed, we need to reset the cell.
    self.pages.take();

    // Next steps are exclusive to books in the library.
    if let Some(id) = self.id() {
      // Remove from library if it was the last page.
      if self.pages().await?.is_empty() {
        return library::remove(&self.app, id).await;
      }

      // Update with a new cover if it was the deleted page.
      if is_cover {
        let first_page = self
          .app
          .book_handle()
          .get_first_page_name(&self.path)
          .await?;

        self
          .app
          .database_handle()
          .update_book_cover(id, &first_page)
          .await?;
      }
    }

    Ok(())
  }
}

impl Drop for ActiveBook {
  fn drop(&mut self) {
    let path = self.path.clone();
    let handle = self.app.book_handle();
    spawn(async move { handle.close(&path).await });

    #[cfg(feature = "tracing")]
    trace!(active_book_drop = %self.path.display());
  }
}

impl fmt::Debug for ActiveBook {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("ActiveBook")
      .field("path", &self.path)
      .field("title", &self.title)
      .finish_non_exhaustive()
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
