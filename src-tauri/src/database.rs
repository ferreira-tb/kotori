mod entities;
pub mod prelude;

use crate::error::Result;
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::sync::Arc;
use tauri::api::path::app_data_dir;
use tauri::Config;

pub async fn connect(config: Arc<Config>) -> Result<DatabaseConnection> {
  let path = app_data_dir(&config).unwrap().join("kotori.db");
  let url = format!("sqlite://{}?mode=rwc", path.to_str().unwrap());

  let options = ConnectOptions::new(&url);
  let conn = Database::connect(options).await?;

  println!("{url}");

  Migrator::up(&conn, None).await?;

  Ok(conn)
}
