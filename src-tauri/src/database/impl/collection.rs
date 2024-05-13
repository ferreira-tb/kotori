use crate::database::entities::{collection, prelude::*};
use crate::prelude::*;
use sea_orm::EntityTrait;

impl Collection {
  pub async fn get_all(app: &AppHandle) -> Result<Vec<collection::Model>> {
    let kotori = app.kotori();
    Self::find()
      .all(&kotori.db)
      .await
      .map_err(Into::into)
  }
}
