use crate::book::Title;
use crate::database::entities::{book, prelude::*};
use crate::prelude::*;
use sea_orm::ActiveValue::Set;
use sea_query::Query;

use sea_orm::{
  ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, IntoActiveModel, PaginatorTrait,
  QueryFilter,
};

impl Book {
  pub async fn count(app: &AppHandle) -> Result<u64> {
    let kotori = app.kotori();
    Self::find()
      .count(&kotori.db)
      .await
      .map_err(Into::into)
  }

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
    let path = path.as_ref();
    let path = path.try_to_str()?;

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

  /// Get a random book from the library.
  /// Will return `None` if the library is empty.
  pub async fn get_random(app: &AppHandle) -> Result<Option<book::Model>> {
    use rand::seq::SliceRandom;
    use rand::thread_rng;

    let kotori = app.kotori();
    let builder = kotori.db.get_database_backend();

    let stmt = Query::select()
      .column(book::Column::Id)
      .from(Book)
      .to_owned();

    let ids = kotori
      .db
      .query_all(builder.build(&stmt))
      .await?
      .into_iter()
      .filter_map(|it| it.try_get::<i32>("", "id").ok())
      .collect_vec();

    let id = {
      let mut rng = thread_rng();
      ids.choose(&mut rng)
    };

    if let Some(id) = id {
      info!("random book selected: {id}");
      Self::get_by_id(app, *id).await.map(Some)
    } else {
      Ok(None)
    }
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
