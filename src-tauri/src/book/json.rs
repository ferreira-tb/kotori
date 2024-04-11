use super::active::ActiveBook;
use super::title::Title;
use crate::database::prelude::*;
use crate::prelude::*;
use crate::reader::ReaderWindow;

pub trait IntoJson {
  async fn into_json(self) -> Result<Json>;
}

pub struct ReaderBook<'a>(pub &'a ActiveBook);

impl<'a> ReaderBook<'a> {
  pub fn from_reader_window(window: &'a ReaderWindow) -> Self {
    Self(&window.book)
  }
}

impl IntoJson for ReaderBook<'_> {
  async fn into_json(self) -> Result<Json> {
    let pages = self
      .0
      .pages_or_try_init()
      .await?
      .keys()
      .copied()
      .sorted_unstable()
      .collect_vec();

    let value = json!({
      "id": self.0.id(),
      "path": self.0.path,
      "title": self.0.title,
      "pages": pages
    });

    Ok(value)
  }
}

pub struct LibraryBook<'a>(pub &'a AppHandle, pub &'a BookModel);

impl IntoJson for LibraryBook<'_> {
  async fn into_json(self) -> Result<Json> {
    let book = ActiveBook::with_model(self.1)?;
    let title = Title::try_from(self.1.path.as_str())?;
    let cover = book.get_cover(self.0).await?;

    let value = json!({
      "id": self.1.id,
      "path": self.1.path,
      "title": title,
      "rating": self.1.rating,
      "cover": Json::from(cover),
    });

    Ok(value)
  }
}
