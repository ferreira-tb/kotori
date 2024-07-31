use crate::book::active::ActiveBook;
use crate::book::cover::Cover;
use crate::book::title::Title;
use crate::database::model::Book;
use crate::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ReaderBook {
  pub id: Option<i32>,
  pub path: PathBuf,
  pub title: Title,
  pub pages: Vec<ReaderBookPage>,
}

impl ReaderBook {
  pub async fn from_active(book: &ActiveBook) -> Result<Self> {
    let id = book.try_id().await.ok();
    let title = book.title.clone();
    let path = book.path.clone();

    let pages = book
      .pages()
      .await?
      .iter()
      .map(|(idx, name)| ReaderBookPage::new((idx, name)))
      .sorted_unstable_by_key(|it| it.index)
      .collect_vec();

    Ok(Self { id, path, title, pages })
  }

  pub async fn from_reader(app: &AppHandle, window_id: u16) -> Result<Self> {
    let windows = app.reader_windows();
    let windows = windows.read().await;
    let book = windows
      .get(&window_id)
      .ok_or_else(|| err!(ReaderWindowNotFound, "{window_id}"))
      .map(|it| &it.book)?;

    #[cfg(feature = "tracing")]
    trace!(?book);

    Self::from_active(book).await
  }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ReaderBookPage {
  pub index: usize,
  pub name: String,
}

impl ReaderBookPage {
  fn new((index, name): (&usize, &str)) -> Self {
    let name = name.to_owned();
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
  pub fn from_model(app: &AppHandle, model: &Book) -> Result<Self> {
    let book = Self {
      id: model.id,
      path: PathBuf::from(&model.path),
      title: Title::new(&model.title),
      rating: u8::try_from(model.rating)?,
      cover: Cover::from_id(app, model.id)?.path_buf(),
    };

    Ok(book)
  }
}
