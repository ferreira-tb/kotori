use crate::book::Title;
use crate::database::prelude::*;
use crate::prelude::*;
use kotori_entity::{book, prelude::*};

pub trait BookExt {
  async fn create<P>(app: &AppHandle, path: P) -> Result<book::Model>
  where
    P: AsRef<Path>;
  async fn get_all(app: &AppHandle) -> Result<Vec<book::Model>>;
  async fn get_by_id(app: &AppHandle, id: i32) -> Result<book::Model>;
  async fn get_by_path(app: &AppHandle, path: impl AsRef<Path>) -> Result<book::Model>;
  async fn get_title(app: &AppHandle, id: i32) -> Result<Title>;
  async fn remove_all(app: &AppHandle) -> Result<()>;
  async fn update_cover<'a, C>(app: &AppHandle, id: i32, cover: C) -> Result<book::Model>
  where
    C: Into<Option<&'a str>>;
  async fn update_rating(app: &AppHandle, id: i32, rating: u8) -> Result<book::Model>;

  /// Get a random book from the library.
  /// Will return `None` if the library is empty.
  async fn get_random(app: &AppHandle) -> Result<Option<book::Model>>;
}

impl BookExt for Book {
  async fn get_all(app: &AppHandle) -> Result<Vec<book::Model>> {
    let kotori = app.kotori();
    Self::find()
      .all(&kotori.db)
      .await
      .map_err(Into::into)
  }

  async fn get_by_id(app: &AppHandle, id: i32) -> Result<book::Model> {
    let kotori = app.kotori();
    Self::find_by_id(id)
      .one(&kotori.db)
      .await?
      .ok_or_else(|| err!(BookNotFound))
  }

  async fn get_by_path(app: &AppHandle, path: impl AsRef<Path>) -> Result<book::Model> {
    let kotori = app.kotori();
    let path = path.try_str()?;
    Self::find()
      .filter(book::Column::Path.eq(path))
      .one(&kotori.db)
      .await?
      .ok_or_else(|| err!(BookNotFound))
  }

  async fn get_title(app: &AppHandle, id: i32) -> Result<Title> {
    let kotori = app.kotori();
    let builder = kotori.db.get_database_backend();

    let stmt = Query::select()
      .column(book::Column::Path)
      .and_where(book::Column::Id.eq(id))
      .from(Book)
      .to_owned();

    kotori
      .db
      .query_one(builder.build(&stmt))
      .await?
      .and_then(|it| it.try_get::<String>("", "path").ok())
      .ok_or_else(|| err!(BookNotFound))
      .and_then(|it| Title::try_from(it.as_str()))
  }

  async fn get_random(app: &AppHandle) -> Result<Option<book::Model>> {
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
      use rand::seq::SliceRandom;
      use rand::thread_rng;

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

  async fn create<P>(app: &AppHandle, path: P) -> Result<book::Model>
  where
    P: AsRef<Path>,
  {
    let path = path.try_string()?;
    let model = book::ActiveModel {
      path: Set(path),
      ..Default::default()
    };

    let kotori = app.kotori();
    Book::insert(model)
      .on_conflict(
        OnConflict::column(book::Column::Path)
          .do_nothing()
          .to_owned(),
      )
      .exec_with_returning(&kotori.db)
      .await
      .map_err(Into::into)
  }

  async fn remove_all(app: &AppHandle) -> Result<()> {
    let kotori = app.kotori();
    let builder = kotori.db.get_database_backend();

    let stmt = Query::delete().from_table(Book).to_owned();
    kotori.db.execute(builder.build(&stmt)).await?;

    Ok(())
  }

  async fn update_cover<'a, C>(app: &AppHandle, id: i32, cover: C) -> Result<book::Model>
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

  async fn update_rating(app: &AppHandle, id: i32, rating: u8) -> Result<book::Model> {
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
