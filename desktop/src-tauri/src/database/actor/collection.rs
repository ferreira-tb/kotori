use crate::database::actor::Db;
use crate::database::model::Collection;
use crate::database::schema::collections::dsl::*;
use crate::utils::result::Result;
use diesel::prelude::*;

pub(super) fn get_all(db: Db) -> Result<Vec<Collection>> {
  collections
    .select(Collection::as_select())
    .load::<Collection>(db)
    .map_err(Into::into)
}
