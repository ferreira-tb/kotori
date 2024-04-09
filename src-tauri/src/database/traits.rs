use super::entities::book;
use super::entities::prelude::*;
use crate::prelude::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

pub mod prelude {
  pub use super::BookExt;
}

pub trait BookExt {
  async fn find_by_path(app: &AppHandle, path: impl AsRef<Path>) -> Result<book::Model>;
}

impl BookExt for Book {
  async fn find_by_path(app: &AppHandle, path: impl AsRef<Path>) -> Result<book::Model> {
    let path = path.as_ref();
    let path = path
      .to_str()
      .ok_or_else(|| err!(InvalidPath, "{}", path.display()))?;

    let kotori = app.state::<Kotori>();
    Book::find()
      .filter(book::Column::Path.eq(path))
      .one(&kotori.db)
      .await?
      .ok_or_else(|| err!(BookNotFound))
  }
}
