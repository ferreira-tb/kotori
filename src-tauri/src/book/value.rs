use super::title::Title;
use super::ActiveBook;
use crate::database::prelude::*;
use crate::prelude::*;
use crate::reader::ReaderWindow;

pub trait IntoValue {
  async fn into_value(self) -> Result<Value>;
}

pub struct ReaderBook<'a>(pub &'a ActiveBook);

impl<'a> ReaderBook<'a> {
  pub fn from_reader_window(window: &'a ReaderWindow) -> Self {
    Self(&window.book)
  }
}

impl IntoValue for ReaderBook<'_> {
  async fn into_value(self) -> Result<Value> {
    let pages = self
      .0
      .pages()
      .await?
      .keys()
      .copied()
      .sorted_unstable()
      .collect_vec();

    let value = json!({
      "path": self.0.path,
      "title": self.0.title,
      "pages": pages
    });

    Ok(value)
  }
}

pub struct LibraryBook<'a>(pub &'a AppHandle, pub &'a BookModel);

impl IntoValue for LibraryBook<'_> {
  async fn into_value(self) -> Result<Value> {
    let active = ActiveBook::with_model(self.1)?;
    let title = Title::try_from(self.1.path.as_str())?;

    let cover = match active.get_cover(self.0).await {
      Ok(cover) => Some(cover),
      Err(Error::CoverNotExtracted) => None,
      Err(err) => return Err(err),
    };

    let value = json!({
      "id": self.1.id,
      "path": self.1.path,
      "title": title,
      "rating": self.1.rating,
      "cover": cover
    });

    Ok(value)
  }
}
