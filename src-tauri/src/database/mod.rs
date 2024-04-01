pub mod entities;
pub mod prelude;

use crate::prelude::*;
use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection};

pub fn connect<M, R>(app: &M) -> Result<DatabaseConnection>
where
  R: Runtime,
  M: Manager<R>,
{
  let path = app
    .path()
    .app_data_dir()
    .unwrap()
    .join("kotori.db");

  async_runtime::block_on(async {
    let url = format!("sqlite://{}?mode=rwc", path.to_str().unwrap());
    let conn = Database::connect(url).await?;

    // Migrator::up(&conn, None).await?;
    Migrator::fresh(&conn).await?;

    Ok(conn)
  })
}
