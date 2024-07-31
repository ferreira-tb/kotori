use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Debug, Serialize, Deserialize)]
#[diesel(table_name = crate::database::schema::collections)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Collection {
  pub id: i32,
  pub name: String,
}
