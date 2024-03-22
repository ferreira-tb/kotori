mod entities;
pub mod prelude;

use crate::error::Result;
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::thread;
use tauri::{Manager, Runtime};

pub fn connect<M, R>(app: &M) -> Result<DatabaseConnection>
where
  R: Runtime,
  M: Manager<R>,
{
  let resolver = app.path();
  let path = resolver.app_data_dir().unwrap().join("kotori.db");
  let url = format!("sqlite://{}?mode=rwc", path.to_str().unwrap());

  let handle = thread::spawn(move || {
    tauri::async_runtime::block_on(async {
      let options = ConnectOptions::new(url);
      let conn = Database::connect(options).await?;

      Migrator::up(&conn, None).await?;

      Ok(conn)
    })
  });

  handle.join().unwrap()
}
