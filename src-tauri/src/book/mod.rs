mod title;
mod value;

pub use title::Title;
pub use value::{IntoValue, LibraryBook, ReaderBook};

use crate::database::prelude::*;
use crate::prelude::*;
use crate::utils::glob;
use crate::utils::OrderedMap;
use natord::compare_ignore_case;
use std::cmp::Ordering;
use std::fs::File;
use std::io::Read;
use tauri_plugin_dialog::{DialogExt, FileDialogBuilder};
use tokio::sync::OnceCell;
use zip::ZipArchive;

type BookHandle = Arc<Mutex<Option<ZipArchive<File>>>>;

pub struct ActiveBook {
  pub path: PathBuf,
  pub title: Title,

  handle: BookHandle,
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

      handle: Arc::new(Mutex::new(None)),
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

  pub async fn id(&self, app: &AppHandle) -> Option<i32> {
    let id = self.id.get_or_try_init(|| async {
      let model = self.get_model(app).await?;
      Ok::<i32, Error>(model.id)
    });

    id.await.ok().copied()
  }

  async fn handle(&self) -> Result<BookHandle> {
    if self.handle.lock().await.is_none() {
      let path = self.path.clone();
      let join: JoinResult<ZipArchive<File>> = async_runtime::spawn_blocking(move || {
        let file = File::open(path)?;
        ZipArchive::new(file).map_err(Into::into)
      });

      let zip = join.await??;
      let mut handle = self.handle.lock().await;
      *handle = Some(zip);
    }

    Ok(Arc::clone(&self.handle))
  }

  /// This should never be called while holding the handle lock.
  /// It'll deadlock if it isn't initialized.
  pub async fn get_pages(&self) -> Result<&OrderedMap<usize, String>> {
    self
      .pages
      .get_or_try_init(|| self.read_pages())
      .await
  }

  async fn read_pages(&self) -> Result<OrderedMap<usize, String>> {
    let handle = self.handle().await?;
    if let Some(ref handle) = *handle.lock().await {
      let globset = glob::book_page();
      let pages = handle
        .file_names()
        .filter(|name| globset.is_match(name))
        .sorted_unstable_by(|a, b| compare_ignore_case(a, b))
        .enumerate()
        .map(|(idx, name)| (idx, name.to_string()))
        .collect();

      return Ok(pages);
    }

    Err(err!(BookHandle))
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

    if let Ok(true) = path.try_exists() {
      return Ok(path);
    }

    self.extract_cover().await
  }

  async fn extract_cover(&self) -> Result<PathBuf> {
    let first = {
      let pages = self.get_pages().await?;
      pages
        .first()
        .map(|(_, name)| name)
        .ok_or_else(|| err!(PageNotFound))?
    };

    let handle = self.handle().await?;
    if let Some(ref mut handle) = *handle.lock().await {
      let _ = handle.by_name(first)?;
    }

    Err(err!(BookHandle))
  }

  pub async fn get_page_as_bytes(&self, page: usize) -> Result<Vec<u8>> {
    let name = self
      .get_pages()
      .await?
      .get(&page)
      .ok_or_else(|| err!(PageNotFound))?;

    let handle = self.handle().await?;
    if let Some(ref mut handle) = *handle.lock().await {
      let mut file = handle.by_name(name)?;
      let size = usize::try_from(file.size()).unwrap_or_default();
      let mut buf = Vec::with_capacity(size);
      file.read_to_end(&mut buf)?;

      return Ok(buf);
    }

    Err(err!(BookHandle))
  }

  async fn get_model(&self, app: &AppHandle) -> Result<BookModel> {
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
