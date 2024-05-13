use crate::book::Title;
use crate::database::entities::{book, prelude::*};
use crate::{prelude::*, utils};
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter};

impl Book {
  pub async fn get_all(app: &AppHandle) -> Result<Vec<book::Model>> {
    let kotori = app.kotori();
    Self::find()
      .all(&kotori.db)
      .await
      .map_err(Into::into)
  }

  pub async fn get_by_id(app: &AppHandle, id: i32) -> Result<book::Model> {
    let kotori = app.kotori();
    Self::find_by_id(id)
      .one(&kotori.db)
      .await?
      .ok_or_else(|| err!(BookNotFound))
  }

  pub async fn get_by_path(app: &AppHandle, path: impl AsRef<Path>) -> Result<book::Model> {
    let kotori = app.kotori();
    let path = utils::path::to_str(path.as_ref())?;

    Self::find()
      .filter(book::Column::Path.eq(path))
      .one(&kotori.db)
      .await?
      .ok_or_else(|| err!(BookNotFound))
  }

  pub async fn get_title(app: &AppHandle, id: i32) -> Result<Title> {
    let book = Self::get_by_id(app, id).await?;
    Title::try_from(book.path.as_str())
  }

  pub async fn update_cover<'a, C>(app: &AppHandle, id: i32, cover: C) -> Result<book::Model>
  where
    C: Into<Option<&'a str>>,
  {
    let book = Self::get_by_id(app, id).await?;
    let mut book = book.into_active_model();

    if let Some(cover) = cover.into() {
      let cover = cover.to_owned();
      book.cover = Set(Some(cover));
    } else {
      book.cover = Set(None);
    }

    let kotori = app.kotori();
    book.update(&kotori.db).await.map_err(Into::into)
  }

  pub async fn update_rating(app: &AppHandle, id: i32, rating: u8) -> Result<book::Model> {
    if rating > 5 {
      bail!(InvalidRating);
    }

    let book = Self::get_by_id(app, id).await?;
    let mut book = book.into_active_model();
    book.rating = Set(i32::from(rating));

    let kotori = app.kotori();
    book.update(&kotori.db).await.map_err(Into::into)
  }
}
