use crate::database::actor::Db;
use crate::database::model::NewFolder;
use crate::database::schema::folders::dsl::*;
use crate::result::Result;
use diesel::prelude::*;
use itertools::Itertools;
use std::path::PathBuf;

pub(super) fn get_all(db: Db) -> Result<Vec<PathBuf>> {
  folders
    .select(path)
    .load::<String>(db)
    .map(|it| it.into_iter().map_into().collect())
    .map_err(Into::into)
}

pub(super) fn is_empty(db: Db) -> Result<bool> {
  use diesel::dsl::count_star;

  folders
    .select(count_star())
    .limit(1)
    .get_result::<i64>(db)
    .map(|it| it == 0)
    .map_err(Into::into)
}

#[cfg(feature = "devtools")]
pub(super) fn remove_all(db: Db) -> Result<()> {
  diesel::delete(folders)
    .execute(db)
    .map(drop)
    .map_err(Into::into)
}

pub(super) fn save_many(db: Db, new_folders: &[NewFolder]) -> Result<()> {
  if new_folders.is_empty() {
    return Ok(());
  }

  diesel::insert_into(folders)
    .values(new_folders)
    .execute(db)
    .map(drop)
    .map_err(Into::into)
}
