use super::active::ActiveBook;
use super::title::Title;
use crate::database::prelude::*;
use crate::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ReaderBook {
  pub id: Option<i32>,
  pub path: PathBuf,
  pub title: Title,
  pub pages: Vec<usize>,
}

impl ReaderBook {
  pub async fn from_active(app: &AppHandle, book: &ActiveBook) -> Result<Self> {
    let id = book.id_or_try_init(app).await.ok();
    let title = book.title.clone();
    let path = book.path.clone();

    let pages = book
      .pages_or_try_init()
      .await?
      .keys()
      .copied()
      .sorted_unstable()
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
pub struct LibraryBook {
  pub id: i32,
  pub path: PathBuf,
  pub title: Title,
  pub rating: u8,
  pub cover: Option<PathBuf>,
}

impl LibraryBook {
  pub async fn from_model(app: &AppHandle, model: &book::Model) -> Result<Self> {
    let book = ActiveBook::with_model(model)?;
    let title = Title::try_from(model.path.as_str())?;
    let rating = u8::try_from(model.rating).unwrap_or(0);

    let cover = book
      .get_cover(app)
      .await?
      .as_path()
      .map(Path::to_path_buf);

    let book = Self {
      id: model.id,
      path: PathBuf::from(&model.path),
      title,
      rating,
      cover,
    };

    Ok(book)
  }
}
