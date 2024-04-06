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
      .get_pages()
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

pub struct LibraryBook<'a>(pub &'a book::Model);

impl IntoValue for LibraryBook<'_> {
  async fn into_value(self) -> Result<Value> {
    let active = ActiveBook::new(&self.0.path)?;
    let cover = active.get_cover().await?;

    let value = json!({
      "id": self.0.id,
      "path": self.0.path,
      "rating": self.0.rating,
      "cover": cover
    });

    Ok(value)
  }
}
