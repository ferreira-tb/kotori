pub mod entities;
mod r#impl;
pub mod prelude;

use crate::prelude::*;
use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection};

pub fn connect(app: &AppHandle) -> Result<DatabaseConnection> {
  let path = app.path().app_local_data_dir().unwrap();

  block_on(async move {
    fs::create_dir_all(&path).await?;

    let path = path.join("kotori.db");
    let url = format!("sqlite://{}?mode=rwc", path.to_str().unwrap());
    let conn = Database::connect(url).await?;

    Migrator::up(&conn, None).await?;
    // Migrator::fresh(&conn).await?;

    Ok(conn)
  })
}
