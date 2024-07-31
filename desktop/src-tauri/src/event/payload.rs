use crate::prelude::*;
use serde::Serialize;

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
  pub fn new(name: &str) -> Self {
    Self { name: name.to_owned() }
  }
}

#[derive(Clone, Debug, Serialize)]
pub struct RatingUpdated {
  pub id: i32,
  pub rating: u8,
}

#[derive(Clone, Debug, Serialize)]
pub struct ReadUpdated {
  pub id: i32,
  pub read: bool,
}
