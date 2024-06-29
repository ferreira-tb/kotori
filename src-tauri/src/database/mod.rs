mod book;
mod collection;

mod prelude {
  pub use sea_orm::error::{DbErr, RuntimeErr};
  pub use sea_orm::ActiveValue::Set;
  pub use sea_query::Query;
  pub use sqlx::error::Error as SqlxError;

  pub use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, IntoActiveModel, QueryFilter,
  };
}

pub use book::BookExt;
pub use collection::CollectionExt;

use crate::prelude::*;
use kotori_migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection};
use tokio::fs;

pub fn connect(app: &AppHandle) -> Result<DatabaseConnection> {
  let path = app.path().app_local_data_dir()?;

  block_on(async move {
    fs::create_dir_all(&path).await?;

    #[cfg(any(debug_assertions, feature = "devtools"))]
    let path = path.join("kotori-dev.db");
    #[cfg(not(any(debug_assertions, feature = "devtools")))]
    let path = path.join("kotori.db");

    let url = format!("sqlite://{}?mode=rwc", path.try_str()?);
    let conn = Database::connect(url).await?;

    #[cfg(not(feature = "ephemeral"))]
    Migrator::up(&conn, None).await?;
    #[cfg(feature = "ephemeral")]
    Migrator::fresh(&conn).await?;

    Ok(conn)
  })
}
