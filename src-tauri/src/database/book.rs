use kotori_entity::book;
use kotori_entity::prelude::*;

use crate::book::{Metadata, Title};
use crate::database::prelude::*;
use crate::database::UniqueViolation;
use crate::prelude::*;

pub trait BookExt {
  async fn get_all(app: &AppHandle) -> Result<Vec<book::Model>>;
  async fn get_by_id(app: &AppHandle, id: i32) -> Result<book::Model>;
  async fn get_by_path(app: &AppHandle, path: impl AsRef<Path>) -> Result<book::Model>;
  async fn get_cover(app: &AppHandle, id: i32) -> Result<Option<String>>;
  async fn get_title(app: &AppHandle, id: i32) -> Result<Title>;
  async fn remove(app: &AppHandle, id: i32) -> Result<()>;
  async fn remove_all(app: &AppHandle) -> Result<()>;

  async fn update_cover<N>(app: &AppHandle, id: i32, name: N) -> Result<book::Model>
  where
    N: AsRef<str>;

  async fn update_rating(app: &AppHandle, id: i32, rating: u8) -> Result<book::Model>;

  /// Get a random book from the library.
  /// Will return `None` if the library is empty.
  async fn get_random(app: &AppHandle) -> Result<Option<book::Model>>;

  fn builder(path: impl AsRef<Path>) -> Builder {
    Builder::new(path)
  }
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

  async fn get_cover(app: &AppHandle, id: i32) -> Result<Option<String>> {
    let kotori = app.kotori();
    let builder = kotori.db.get_database_backend();

    let stmt = Query::select()
      .column(book::Column::Cover)
      .and_where(book::Column::Id.eq(id))
      .from(Book)
      .to_owned();

    kotori
      .db
      .query_one(builder.build(&stmt))
      .await?
      .ok_or_else(|| err!(BookNotFound))
      .map(|it| it.try_get::<String>("", "cover").ok())
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
    let database = kotori.db.get_database_backend();

    let stmt = Query::select()
      .column(book::Column::Id)
      .from(Book)
      .to_owned();

    let ids = kotori
      .db
      .query_all(database.build(&stmt))
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

  async fn remove(app: &AppHandle, id: i32) -> Result<()> {
    let kotori = app.kotori();
    Book::delete_by_id(id)
      .exec(&kotori.db)
      .await
      .map(|_| ())
      .map_err(Into::into)
  }

  async fn remove_all(app: &AppHandle) -> Result<()> {
    let kotori = app.kotori();
    let database = kotori.db.get_database_backend();

    let stmt = Query::delete().from_table(Book).to_owned();
    kotori.db.execute(database.build(&stmt)).await?;

    Ok(())
  }

  async fn update_cover<N>(app: &AppHandle, id: i32, name: N) -> Result<book::Model>
  where
    N: AsRef<str>,
  {
    let book = Self::get_by_id(app, id).await?;
    let mut book = book.into_active_model();
    book.cover = Set(name.as_ref().to_owned());

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

#[derive(Debug)]
pub struct Builder {
  path: PathBuf,
  title: Option<Title>,
  rating: Option<u8>,
  cover: Option<String>,
}

impl Builder {
  pub fn new(path: impl AsRef<Path>) -> Self {
    let path = path.as_ref().to_path_buf();
    Self {
      path,
      title: None,
      rating: None,
      cover: None,
    }
  }

  pub fn cover(mut self, cover: String) -> Self {
    self.cover = Some(cover);
    self
  }

  pub fn title(mut self, title: Title) -> Self {
    self.title = Some(title);
    self
  }

  pub fn metadata(mut self, mut metadata: Metadata) -> Self {
    if let Some(title) = metadata.title.take() {
      self.title = Some(title);
    }

    if matches!(metadata.rating, Some(it) if it <= 5) {
      self.rating = metadata.rating.take();
    }

    if let Some(cover) = metadata.cover.take() {
      self.cover = Some(cover);
    }

    self
  }

  pub async fn build(mut self, app: &AppHandle) -> Result<Option<book::Model>> {
    let path = self.path.try_string()?;
    let title = match self.title {
      Some(it) => it.to_string(),
      None => Title::try_from(&self.path)?.to_string(),
    };

    let cover = match self.cover.take() {
      Some(cover) => cover,
      None => {
        app
          .book_handle()
          .get_first_page_name(&self.path)
          .await?
      }
    };

    let model = book::ActiveModel {
      path: Set(path),
      title: Set(title),
      rating: self.rating.map_or(NotSet, |it| Set(it.into())),
      cover: Set(cover),
      ..Default::default()
    };

    let kotori = app.kotori();
    let result = Book::insert(model)
      .exec_with_returning(&kotori.db)
      .await;

    if matches!(&result, Err(e) if e.is_unique_violation()) {
      return Ok(None);
    }

    result.map(Some).map_err(Into::into)
  }
}
