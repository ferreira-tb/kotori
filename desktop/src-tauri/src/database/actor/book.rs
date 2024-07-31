use crate::bail;
use crate::book::Title;
use crate::database::actor::Db;
use crate::database::model::{Book, NewBook};
use crate::database::schema::books::dsl::*;
use crate::path::PathExt;
use crate::result::Result;
use diesel::prelude::*;
use std::path::{Path, PathBuf};

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

pub(super) fn save(db: Db, new_book: &NewBook) -> Result<Book> {
  diesel::insert_into(books)
    .values(new_book)
    .returning(Book::as_returning())
    .get_result(db)
    .map_err(Into::into)
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
