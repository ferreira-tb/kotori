use crate::bail;
use crate::book::Title;
use crate::database::actor::Db;
use crate::database::model::{Book, NewBook};
use crate::database::schema::books::dsl::*;
use crate::path::PathExt;
use crate::result::Result;
use diesel::prelude::*;
use std::path::{Path, PathBuf};
#[cfg(feature = "tracing")]
use {
  std::time::Instant,
  tracing::{instrument, trace},
};

pub(super) fn get_all(db: Db) -> Result<Vec<Book>> {
  books
    .select(Book::as_select())
    .load::<Book>(db)
    .map_err(Into::into)
}

pub(super) fn get_by_id(db: Db, book_id: i32) -> Result<Book> {
  books
    .find(book_id)
    .select(Book::as_select())
    .first::<Book>(db)
    .map_err(Into::into)
}

pub(super) fn get_by_path(db: Db, book_path: &Path) -> Result<Book> {
  let book_path = book_path.try_str()?;
  books
    .filter(path.eq(book_path))
    .select(Book::as_select())
    .first::<Book>(db)
    .map_err(Into::into)
}

pub(super) fn get_cover(db: Db, book_id: i32) -> Result<String> {
  books
    .find(book_id)
    .select(cover)
    .first::<String>(db)
    .map_err(Into::into)
}

pub(super) fn get_path(db: Db, book_id: i32) -> Result<PathBuf> {
  books
    .find(book_id)
    .select(path)
    .first::<String>(db)
    .map(PathBuf::from)
    .map_err(Into::into)
}

pub(super) fn get_title(db: Db, book_id: i32) -> Result<Title> {
  books
    .find(book_id)
    .select(title)
    .first::<String>(db)
    .map(Title::new)
    .map_err(Into::into)
}

#[cfg_attr(feature = "tracing", instrument(skip(db), level = "trace"))]
pub(super) fn has_path(db: Db, book_path: &Path) -> Result<bool> {
  use diesel::dsl::count_star;

  #[cfg(feature = "tracing")]
  let start = Instant::now();

  let book_path = book_path.try_str()?;
  let has = books
    .select(count_star())
    .filter(path.eq(book_path))
    .limit(1)
    .get_result::<i64>(db)
    .map(|count| count > 0)?;

  #[cfg(feature = "tracing")]
  trace!(has_path = has, "path checked in {:?}", start.elapsed());

  Ok(has)
}

pub(super) fn is_empty(db: Db) -> Result<bool> {
  use diesel::dsl::count_star;

  books
    .select(count_star())
    .limit(1)
    .get_result::<i64>(db)
    .map(|count| count == 0)
    .map_err(Into::into)
}

pub(super) fn random(db: Db) -> Result<Option<Book>> {
  use rand::seq::SliceRandom;
  use rand::thread_rng;

  let ids = books.select(id).load::<i32>(db)?;
  match ids.choose(&mut thread_rng()) {
    Some(book_id) => get_by_id(db, *book_id).map(Some),
    None => Ok(None),
  }
}

pub(super) fn remove(db: Db, book_id: i32) -> Result<()> {
  diesel::delete(books.find(book_id))
    .execute(db)
    .map(drop)
    .map_err(Into::into)
}

#[cfg(feature = "devtools")]
pub(super) fn remove_all(db: Db) -> Result<()> {
  diesel::delete(books)
    .execute(db)
    .map(drop)
    .map_err(Into::into)
}

#[cfg_attr(feature = "tracing", instrument(skip(db), level = "trace"))]
pub(super) fn save(db: Db, new_book: &NewBook) -> Result<Book> {
  #[cfg(feature = "tracing")]
  let start = Instant::now();

  let book = diesel::insert_into(books)
    .values(new_book)
    .returning(Book::as_returning())
    .get_result(db)?;

  #[cfg(feature = "tracing")]
  trace!(book_id = book.id, "book saved in {:?}", start.elapsed());

  Ok(book)
}

pub(super) fn update_cover(db: Db, book_id: i32, book_cover: &str) -> Result<Book> {
  diesel::update(books.find(book_id))
    .set(cover.eq(book_cover))
    .returning(Book::as_returning())
    .get_result(db)
    .map_err(Into::into)
}

pub(super) fn update_rating(db: Db, book_id: i32, book_rating: u8) -> Result<Book> {
  if book_rating > 5 {
    bail!(InvalidRating);
  }

  diesel::update(books.find(book_id))
    .set(rating.eq(i32::from(book_rating)))
    .returning(Book::as_returning())
    .get_result(db)
    .map_err(Into::into)
}

pub(super) fn update_read(db: Db, book_id: i32, yes: bool) -> Result<Book> {
  diesel::update(books.find(book_id))
    .set(read.eq(yes))
    .returning(Book::as_returning())
    .get_result(db)
    .map_err(Into::into)
}
