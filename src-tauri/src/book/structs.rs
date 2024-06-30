use kotori_entity::book;
use serde::Serialize;

use crate::book::active::ActiveBook;
use crate::book::cover::Cover;
use crate::book::title::Title;
use crate::prelude::*;
use crate::window::WindowManager;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ReaderBook {
  pub id: Option<i32>,
  pub path: PathBuf,
  pub title: Title,
  pub pages: Vec<ReaderBookPage>,
}

impl ReaderBook {
  pub async fn from_active(app: &AppHandle, book: &ActiveBook) -> Result<Self> {
    let id = book.try_id(app).await.ok();
    let title = book.title.clone();
    let path = book.path.clone();

    let pages = book
      .pages()
      .await?
      .iter()
      .map(ReaderBookPage::new)
      .sorted_unstable_by_key(|it| it.index)
      .collect_vec();

    Ok(Self { id, path, title, pages })
  }

  pub async fn from_reader(app: &AppHandle, window_id: u16) -> Result<Self> {
    let windows = app.reader_windows();
    let windows = windows.read().await;
    let book = windows
      .get(&window_id)
      .ok_or_else(|| err!(WindowNotFound, "{window_id}"))
      .map(|it| &it.book)?;

    Self::from_active(app, book).await
  }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ReaderBookPage {
  pub index: usize,
  pub name: String,
}

impl ReaderBookPage {
  fn new((index, name): (&usize, impl AsRef<str>)) -> Self {
    let name = name.as_ref().to_owned();
    ReaderBookPage { index: *index, name }
  }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct LibraryBook {
  pub id: i32,
  pub path: PathBuf,
  pub title: Title,
  pub rating: u8,
  pub cover: Option<PathBuf>,
}

impl LibraryBook {
  pub fn from_model(app: &AppHandle, model: &book::Model) -> Result<Self> {
    let book = Self {
      id: model.id,
      path: PathBuf::from(&model.path),
      title: Title::try_from(model.path.as_str())?,
      rating: u8::try_from(model.rating)?,
      cover: Cover::from_id(app, model.id)?.path_buf(),
    };

    Ok(book)
  }
}
