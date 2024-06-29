use crate::database::{prelude::*, UniqueViolation};
use crate::prelude::*;
use kotori_entity::{folder, prelude::*};

pub trait FolderExt {
  async fn create(app: &AppHandle, path: PathBuf) -> Result<()>;
  async fn get_all(app: &AppHandle) -> Result<Vec<PathBuf>>;
}

impl FolderExt for Folder {
  async fn get_all(app: &AppHandle) -> Result<Vec<PathBuf>> {
    let kotori = app.kotori();
    let builder = kotori.db.get_database_backend();

    let stmt = Query::select()
      .column(folder::Column::Path)
      .from(Folder)
      .to_owned();

    let folders = kotori
      .db
      .query_all(builder.build(&stmt))
      .await?
      .into_iter()
      .filter_map(|it| it.try_get::<String>("", "path").ok())
      .map_into()
      .collect();

    Ok(folders)
  }

  async fn create(app: &AppHandle, path: PathBuf) -> Result<()> {
    let path = path.try_string()?;
    let model = folder::ActiveModel {
      path: Set(path),
      ..Default::default()
    };

    let kotori = app.kotori();
    let result = Folder::insert(model).exec(&kotori.db).await;

    if matches!(&result, Err(e) if e.is_unique_violation()) {
      return Ok(());
    }

    result.map(|_| ()).map_err(Into::into)
  }
}
