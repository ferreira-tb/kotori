use diesel::prelude::*;
use serde::{Deserialize, Serialize};

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
