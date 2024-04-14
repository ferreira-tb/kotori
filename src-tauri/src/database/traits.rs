use super::entities::book;
use super::entities::prelude::*;
use crate::book::Title;
use crate::prelude::*;
use crate::utils;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};

pub mod prelude {
  pub use super::BookExt;
}

pub trait BookExt {
  async fn get_by_id(app: &AppHandle, id: i32) -> Result<book::Model> {
    let kotori = app.kotori();
    Book::find_by_id(id)
      .one(&kotori.db)
      .await?
      .ok_or_else(|| err!(BookNotFound))
  }

  async fn get_by_path(app: &AppHandle, path: impl AsRef<Path>) -> Result<book::Model> {
    let kotori = app.kotori();
    let path = utils::path::to_str(path.as_ref())?;

    Book::find()
      .filter(book::Column::Path.eq(path))
      .one(&kotori.db)
      .await?
      .ok_or_else(|| err!(BookNotFound))
  }

  async fn get_title(app: &AppHandle, id: i32) -> Result<Title> {
    let book = Self::get_by_id(app, id).await?;
    Title::try_from(book.path.as_str())
  }

  async fn update_cover<C>(app: &AppHandle, id: i32, cover: Option<C>) -> Result<book::Model>
  where
    C: AsRef<Path>,
  {
    let book = Self::get_by_id(app, id).await?;
    let mut book: book::ActiveModel = book.into();

    if let Some(cover) = cover {
      let cover = utils::path::to_str(cover.as_ref())?;
      book.cover = Set(Some(cover.into()));
    } else {
      book.cover = Set(None);
    }

    let kotori = app.kotori();
    book.update(&kotori.db).await.map_err(Into::into)
  }

  async fn update_rating(app: &AppHandle, id: i32, rating: u8) -> Result<book::Model> {
    if rating > 5 {
      bail!(InvalidRating);
    }

    let book = Self::get_by_id(app, id).await?;
    let mut book: book::ActiveModel = book.into();
    book.rating = Set(i32::from(rating));

    let kotori = app.kotori();
    book.update(&kotori.db).await.map_err(Into::into)
  }
}

impl BookExt for Book {}
