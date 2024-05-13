pub mod entities;
mod r#impl;
pub mod prelude;

use crate::prelude::*;
use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection};
use tokio::fs;

pub fn connect(app: &AppHandle) -> Result<DatabaseConnection> {
  let path = app.path().app_local_data_dir().unwrap();

  block_on(async move {
    fs::create_dir_all(&path).await?;

    #[cfg(any(debug_assertions, feature = "devtools"))]
    let path = path.join("kotori-dev.db");
    #[cfg(not(any(debug_assertions, feature = "devtools")))]
    let path = path.join("kotori.db");

    let url = format!("sqlite://{}?mode=rwc", path.to_str().unwrap());
    let conn = Database::connect(url).await?;

    #[cfg(not(feature = "fresh"))]
    Migrator::up(&conn, None).await?;
    #[cfg(feature = "fresh")]
    Migrator::fresh(&conn).await?;

    Ok(conn)
  })
}
