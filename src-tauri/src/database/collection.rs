use kotori_entity::{collection, prelude::*};

use crate::{database::prelude::*, prelude::*};

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
