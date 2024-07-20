use crate::database::prelude::*;
#[cfg(any(debug_assertions, feature = "devtools"))]
use crate::database::UniqueViolation;
use crate::prelude::*;
use kotori_entity::folder;
use kotori_entity::prelude::*;

pub trait FolderExt {
  async fn create_many<I>(app: &AppHandle, folders: I) -> Result<()>
  where
    I: IntoIterator<Item = PathBuf>;

  async fn get_all(app: &AppHandle) -> Result<Vec<PathBuf>>;

  #[cfg(any(debug_assertions, feature = "devtools"))]
  async fn create(app: &AppHandle, folder: impl AsRef<Path>) -> Result<Option<folder::Model>>;
  #[cfg(any(debug_assertions, feature = "devtools"))]
  async fn remove_all(app: &AppHandle) -> Result<()>;
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

  #[cfg(any(debug_assertions, feature = "devtools"))]
  async fn create(app: &AppHandle, folder: impl AsRef<Path>) -> Result<Option<folder::Model>> {
    let path = folder.try_string()?;
    let model = folder::ActiveModel {
      path: Set(path),
      ..Default::default()
    };

    let kotori = app.kotori();
    let result = Folder::insert(model)
      .exec_with_returning(&kotori.db)
      .await;

    if matches!(&result, Err(e) if e.is_unique_violation()) {
      return Ok(None);
    }

    result.map(Some).map_err(Into::into)
  }

  async fn create_many<I>(app: &AppHandle, folders: I) -> Result<()>
  where
    I: IntoIterator<Item = PathBuf>,
  {
    let models = folders
      .into_iter()
      .filter_map(|folder| {
        let path = folder.try_string().ok()?;
        let model = folder::ActiveModel {
          path: Set(path),
          ..Default::default()
        };

        Some(model)
      })
      .collect_vec();

    let kotori = app.kotori();
    Folder::insert_many(models)
      .on_conflict(
        OnConflict::column(folder::Column::Path)
          .do_nothing()
          .to_owned(),
      )
      .on_empty_do_nothing()
      .exec(&kotori.db)
      .await
      .map(|_| ())
      .map_err(Into::into)
  }

  #[cfg(any(debug_assertions, feature = "devtools"))]
  async fn remove_all(app: &AppHandle) -> Result<()> {
    let kotori = app.kotori();
    let database = kotori.db.get_database_backend();

    let stmt = Query::delete().from_table(Folder).to_owned();
    kotori.db.execute(database.build(&stmt)).await?;

    Ok(())
  }
}
