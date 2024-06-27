pub mod entities;
mod r#impl;

pub mod prelude {
  pub use super::entities::book;
  pub use super::entities::prelude::*;
  pub use sea_orm::sea_query::OnConflict;
  pub use sea_orm::ActiveValue::Set;
  pub use sea_orm::{ActiveModelTrait, ConnectionTrait, EntityTrait};
}

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
