use crate::database::prelude::*;
use crate::prelude::*;
use kotori_entity::collection;
use kotori_entity::prelude::*;

pub trait CollectionExt {
  async fn get_all(app: &AppHandle) -> Result<Vec<collection::Model>>;
}

impl CollectionExt for Collection {
  async fn get_all(app: &AppHandle) -> Result<Vec<collection::Model>> {
    let kotori = app.kotori();
    Self::find()
      .all(&kotori.db)
      .await
      .map_err(Into::into)
  }
}
