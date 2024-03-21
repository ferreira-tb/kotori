use crate::error::Result;
use globset::{Glob, GlobBuilder, GlobSet, GlobSetBuilder};
use serde::ser::Serializer;
use serde::Serialize;
use std::path::Path;
use tempfile::tempdir_in;

#[derive(Debug)]
pub struct TempDir(tempfile::TempDir);

impl TempDir {
  pub fn path(&self) -> &Path {
    self.0.path()
  }
}

impl TryFrom<&Path> for TempDir {
  type Error = crate::error::Error;

  fn try_from(path: &Path) -> Result<Self> {
    Ok(Self(tempdir_in(path)?))
  }
}

impl Serialize for TempDir {
  fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let path = self.0.path();
    let path = path.to_str().ok_or_else(|| {
      let err = format!("invalid path: {path:?}");
      serde::ser::Error::custom(err)
    })?;

    serializer.serialize_str(path)
  }
}

pub fn img_glob(glob: &str) -> Result<Glob> {
  GlobBuilder::new(glob)
    .case_insensitive(true)
    .build()
    .map_err(Into::into)
}

pub fn img_globset() -> Result<GlobSet> {
  GlobSetBuilder::new()
    .add(img_glob("*.gif")?)
    .add(img_glob("*.jpg")?)
    .add(img_glob("*.jpeg")?)
    .add(img_glob("*.png")?)
    .add(img_glob("*.webp")?)
    .build()
    .map_err(Into::into)
}
