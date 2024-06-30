use serde::Serialize;

use crate::prelude::*;

#[derive(Clone, Debug, Serialize)]
pub struct BookRemoved {
  pub id: i32,
}

#[derive(Clone, Debug, Serialize)]
pub struct CoverExtracted {
  pub id: i32,
  pub path: String,
}

impl CoverExtracted {
  pub fn new(id: i32, path: impl AsRef<Path>) -> Result<Self> {
    path.try_string().map(|path| Self { id, path })
  }
}

#[derive(Clone, Debug, Serialize)]
pub struct PageDeleted {
  pub name: String,
}

impl PageDeleted {
  pub fn new(name: impl AsRef<str>) -> Self {
    let name = name.as_ref().to_owned();
    Self { name }
  }
}

#[derive(Clone, Debug, Serialize)]
pub struct RatingUpdated {
  pub id: i32,
  pub rating: u8,
}
