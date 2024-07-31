use super::title::Title;
use crate::database::model::Book;
use crate::prelude::*;
use crate::VERSION;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct Metadata {
  pub title: Option<Title>,
  pub cover: Option<String>,
  pub rating: Option<u8>,
  pub read: Option<bool>,

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
    
    let metadata = Builder::new(&book.path)
      .title(title)
      .cover(&book.cover)
      .rating(rating)
      .read(book.read)
      .build();

    Ok(metadata)
  }
}

#[derive(Debug)]
pub struct Builder {
  title: Option<Title>,
  cover: Option<String>,
  rating: u8,
  read: bool,
}

impl Builder {
  pub fn new(path: impl AsRef<Path>) -> Self {
    let path = path.as_ref();
    let title = Title::try_from(path).ok();

    Self {
      title,
      rating: 0,
      cover: None,
      read: false,
    }
  }

  pub fn cover(mut self, cover: impl AsRef<str>) -> Self {
    let cover = cover.as_ref().to_owned();
    self.cover = Some(cover);
    self
  }

  pub fn rating(mut self, rating: u8) -> Self {
    self.rating = rating;
    self
  }

  pub fn read(mut self, read: bool) -> Self {
    self.read = read;
    self
  }

  pub fn title(mut self, title: Title) -> Self {
    self.title = Some(title);
    self
  }

  pub fn build(self) -> Metadata {
    let version = Version::parse(VERSION).unwrap();
    Metadata {
      title: self.title,
      cover: self.cover,
      rating: Some(self.rating),
      read: Some(self.read),
      version: Some(version),
    }
  }
}
