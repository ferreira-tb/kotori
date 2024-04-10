use super::entities::book;
use super::entities::prelude::*;
use crate::prelude::*;
use crate::utils;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};

pub mod prelude {
  pub use super::BookExt;
}

pub trait BookExt {
  async fn find_by_path(app: &AppHandle, path: impl AsRef<Path>) -> Result<book::Model> {
    let path = utils::path::to_str(path.as_ref())?;
    let kotori = app.state::<Kotori>();

    Book::find()
      .filter(book::Column::Path.eq(path))
      .one(&kotori.db)
      .await?
      .ok_or_else(|| err!(BookNotFound))
  }

  async fn update_book_cover(app: &AppHandle, id: i32, cover: impl AsRef<Path>) -> Result<()> {
    let cover = utils::path::to_str(cover.as_ref())?;
    let kotori = app.state::<Kotori>();

    let book = Book::find_by_id(id)
      .one(&kotori.db)
      .await?
      .ok_or_else(|| err!(BookNotFound))?;

    let mut book: book::ActiveModel = book.into();
    book.cover = Set(Some(cover.into()));
    book.update(&kotori.db).await?;

    Ok(())
  }
}

impl BookExt for Book {}
