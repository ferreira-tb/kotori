use crate::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct BookRemoved {
  pub id: i32,
}

impl BookRemoved {
  pub fn new(id: i32) -> Self {
    Self { id }
  }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct CoverExtracted {
  pub id: i32,
  pub path: String,
}

impl CoverExtracted {
  pub fn new(id: i32, path: impl AsRef<Path>) -> Result<Self> {
    path.try_to_string().map(|path| Self { id, path })
  }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct RatingUpdated {
  pub id: i32,
  pub rating: u8,
}

impl RatingUpdated {
  pub fn new(id: i32, rating: u8) -> Self {
    Self { id, rating }
  }
}
