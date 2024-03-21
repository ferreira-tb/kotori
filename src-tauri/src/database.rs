mod entities;
pub mod prelude;

use crate::error::Result;
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::thread;
use tauri::api::path::app_data_dir;
use tauri::Config;

pub fn connect(config: &Config) -> Result<DatabaseConnection> {
  let path = app_data_dir(config).unwrap().join("kotori.db");
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
