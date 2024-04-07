mod handle;
mod title;
mod value;

pub use title::Title;
pub use value::{IntoValue, LibraryBook, ReaderBook};

use crate::database::prelude::*;
use crate::prelude::*;
use crate::utils::OrderedMap;
use handle::Handle;
use natord::compare_ignore_case;
use std::cmp::Ordering;
use tauri_plugin_dialog::{DialogExt, FileDialogBuilder};
use tokio::sync::OnceCell;

pub struct ActiveBook {
  pub path: PathBuf,
  pub title: Title,

  handle: OnceCell<Handle>,
  pages: OnceCell<OrderedMap<usize, String>>,

  /// Book id in the database.
  id: OnceCell<i32>,
}

impl ActiveBook {
  pub fn new(path: impl AsRef<Path>) -> Result<Self> {
    let path = path.as_ref();
    let book = Self {
      path: path.to_owned(),
      title: Title::try_from(path)?,

      handle: OnceCell::new(),
      pages: OnceCell::new(),

      id: OnceCell::new(),
    };

    Ok(book)
  }

  pub fn with_model(model: &BookModel) -> Result<Self> {
    let book = Self::new(&model.path)?;
    book.id.set(model.id).ok();

    Ok(book)
  }

  async fn handle(&self) -> Result<&Handle> {
    let handle = self.handle.get_or_try_init(|| async {
      let handle = Handle::new(&self.path).await?;
      Ok::<Handle, Error>(handle)
    });

    handle.await
  }

  pub async fn pages(&self) -> Result<&OrderedMap<usize, String>> {
    let pages = self.pages.get_or_try_init(|| async {
      let handle = self.handle().await?;
      let pages = handle.pages().await;
      Ok::<OrderedMap<usize, String>, Error>(pages)
    });

    pages.await
  }

  pub async fn id(&self, app: &AppHandle) -> Option<i32> {
    let id = self.id.get_or_try_init(|| async {
      let model = self.model(app).await?;
      Ok::<i32, Error>(model.id)
    });

    id.await.ok().copied()
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

  pub async fn open_book_from_dialog(app: &AppHandle) -> Result<()> {
    let books = Self::show_dialog(app).await?;

    if !books.is_empty() {
      let kotori = app.state::<Kotori>();
      let reader = kotori.reader.read().await;
      return reader.open_many(books).await.map_err(Into::into);
    }

    Ok(())
  }

  pub async fn get_cover(&self, app: &AppHandle) -> Result<PathBuf> {
    let id = self
      .id(app)
      .await
      .ok_or_else(|| err!(BookNotFound))?;

    let path = app
      .path()
      .app_cache_dir()?
      .join(format!("covers/{id}"));

    if path.try_exists()? {
      return Ok(path);
    }

    Err(err!(CoverNotExtracted))
  }

  #[allow(dead_code)]
  async fn extract_cover(&self, path: impl AsRef<Path>) -> Result<()> {
    let first = self
      .pages()
      .await?
      .first()
      .map(|(_, name)| name)
      .ok_or_else(|| err!(PageNotFound))?;

    let handle = self.handle().await?;
    let page = handle.by_name(first).await?;

    if let Some(parent_dir) = path.as_ref().parent() {
      fs::create_dir_all(parent_dir).await?;
    }

    fs::write(&path, page).await.map_err(Into::into)
  }

  pub async fn get_page_as_bytes(&self, page: usize) -> Result<Vec<u8>> {
    let name = self
      .pages()
      .await?
      .get(&page)
      .ok_or_else(|| err!(PageNotFound))?;

    let handle = self.handle().await?;
    handle.by_name(name).await
  }

  async fn model(&self, app: &AppHandle) -> Result<BookModel> {
    let kotori = app.state::<Kotori>();
    let path = self
      .path
      .to_str()
      .ok_or_else(|| err!(InvalidBookPath, "{}", self.path.display()))?;

    Book::find()
      .filter(BookColumn::Path.eq(path))
      .one(&kotori.db)
      .await?
      .ok_or_else(|| err!(BookNotFound))
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
