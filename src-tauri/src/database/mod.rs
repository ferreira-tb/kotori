mod book;
mod collection;
mod folder;

mod prelude {
  pub use sea_query::{OnConflict, Query};

  pub use sea_orm::{
    ActiveModelTrait,
    ActiveValue::{NotSet, Set},
    ColumnTrait, ConnectionTrait, EntityTrait, IntoActiveModel, QueryFilter,
  };
}

pub use book::BookExt;
pub use collection::CollectionExt;
pub use folder::FolderExt;

use crate::prelude::*;
use kotori_migration::{Migrator, MigratorTrait};
use sea_orm::error::{DbErr, RuntimeErr};
use sea_orm::{Database, DatabaseConnection};
use sqlx::error::Error as SqlxError;
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

    Migrator::up(&conn, None).await?;

    Ok(conn)
  })
}

trait UniqueViolation: std::error::Error {
  fn is_unique_violation(&self) -> bool;
}

impl UniqueViolation for DbErr {
  fn is_unique_violation(&self) -> bool {
    if let DbErr::Exec(runtime_err) = self
      && let RuntimeErr::SqlxError(sqlx_err) = runtime_err
      && let SqlxError::Database(db_err) = sqlx_err
      && db_err.is_unique_violation()
    {
      warn!(error = ?db_err);
      true
    } else {
      false
    }
  }
}
