use super::title::Title;
use crate::prelude::*;
use crate::VERSION;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::path::Path;
use crate::database::model::Book;

#[derive(Debug, Deserialize, Serialize)]
pub struct Metadata {
  pub title: Option<Title>,
  pub rating: Option<u8>,
  pub cover: Option<String>,

  /// Kotori version.
  pub version: Option<Version>,
}

impl Metadata {
  pub fn builder(path: impl AsRef<Path>) -> Builder {
    Builder::new(path)
  }
}

impl TryFrom<&Book> for Metadata {
  type Error = crate::error::Error;

  fn try_from(book: &Book) -> Result<Self> {
    let title = Title::new(&book.title);
    let rating = u8::try_from(book.rating)?;
    Builder::new(&book.path)
      .title(title)
      .cover(&book.cover)
      .rating(rating)
      .map(Builder::build)
  }
}

#[derive(Debug)]
pub struct Builder {
  title: Option<Title>,
  rating: Option<u8>,
  cover: Option<String>,
}

impl Builder {
  pub fn new(path: impl AsRef<Path>) -> Self {
    let path = path.as_ref();
    let title = Title::try_from(path).ok();
    Self { title, rating: Some(0), cover: None }
  }

  pub fn cover(mut self, cover: impl AsRef<str>) -> Self {
    let cover = cover.as_ref().to_owned();
    self.cover = Some(cover);
    self
  }

  pub fn rating(mut self, rating: u8) -> Result<Self> {
    if rating > 5 {
      bail!(InvalidRating);
    }

    self.rating = Some(rating);
    Ok(self)
  }

  pub fn title(mut self, title: Title) -> Self {
    self.title = Some(title);
    self
  }

  pub fn build(self) -> Metadata {
    let version = Version::parse(VERSION).unwrap();
    Metadata {
      title: self.title,
      rating: self.rating,
      cover: self.cover,
      version: Some(version),
    }
  }
}
