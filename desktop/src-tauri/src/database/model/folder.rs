use crate::path::PathExt;
use crate::result::Result;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Queryable, Selectable, Debug, Serialize, Deserialize)]
#[diesel(table_name = crate::database::schema::folders)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Folder {
  pub id: i32,
  pub path: String,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::database::schema::folders)]
pub struct NewFolder {
  pub path: String,
}

impl TryFrom<PathBuf> for NewFolder {
  type Error = crate::error::Error;

  fn try_from(path: PathBuf) -> Result<Self> {
    path.try_string().map(|path| Self { path })
  }
}
