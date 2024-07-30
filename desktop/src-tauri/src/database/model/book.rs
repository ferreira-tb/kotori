use crate::book::{Metadata, Title};
use crate::utils::manager::ManagerExt;
use crate::utils::path::PathExt;
use crate::utils::result::Result;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::AppHandle;

#[derive(Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::database::schema::books)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Book {
  pub id: i32,
  pub path: String,
  pub title: String,
  pub cover: String,
  pub rating: i32,
}

impl Book {
  pub fn builder(path: impl AsRef<Path>) -> BookBuilder {
    BookBuilder::new(path)
  }

  pub async fn save_as_metadata(&self, app: &AppHandle) -> Result<()> {
    let metadata = Metadata::try_from(self)?;
    app
      .book_handle()
      .set_metadata(&self.path, metadata)
      .await
  }
}

#[derive(Insertable)]
#[diesel(table_name = crate::database::schema::books)]
pub struct NewBook {
  path: String,
  title: String,
  cover: String,
  rating: i32,
}

#[derive(Debug)]
pub struct BookBuilder {
  path: PathBuf,
  title: Option<Title>,
  rating: Option<u8>,
  cover: Option<String>,
}

impl BookBuilder {
  pub fn new(path: impl AsRef<Path>) -> Self {
    let path = path.as_ref().to_path_buf();
    Self {
      path,
      title: None,
      rating: None,
      cover: None,
    }
  }

  pub fn cover(mut self, cover: String) -> Self {
    self.cover = Some(cover);
    self
  }

  pub fn title(mut self, title: Title) -> Self {
    self.title = Some(title);
    self
  }

  pub fn metadata(mut self, mut metadata: Metadata) -> Self {
    if metadata.title.is_some() {
      self.title = metadata.title.take();
    }

    if metadata.rating.is_some_and(|it| it <= 5) {
      self.rating = metadata.rating.take();
    }

    if metadata.cover.is_some() {
      self.cover = metadata.cover.take();
    }

    self
  }

  pub async fn build(mut self, app: &AppHandle) -> Result<NewBook> {
    let path = self.path.try_string()?;
    let title = match self.title {
      Some(it) => it.to_string(),
      None => Title::try_from(&self.path)?.to_string(),
    };

    let cover = match self.cover.take() {
      Some(cover) => cover,
      None => {
        app
          .book_handle()
          .get_first_page_name(&self.path)
          .await?
      }
    };

    Ok(NewBook {
      path,
      title,
      cover,
      rating: self.rating.map_or(0, Into::into),
    })
  }
}
