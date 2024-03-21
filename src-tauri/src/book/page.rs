use crate::error::{Error, Result};
use serde::Serialize;
use std::cmp::Ordering;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Page {
  pub path: PathBuf,
  filename: String,
}

impl Page {
  pub fn new(path: PathBuf) -> Result<Self> {
    let filename = path
      .file_name()
      .ok_or_else(|| Error::InvalidPage("file name not found".into()))?
      .to_string_lossy()
      .into_owned();

    let page = Self { path, filename };
    Ok(page)
  }
}

impl TryFrom<PathBuf> for Page {
  type Error = crate::error::Error;

  fn try_from(path: PathBuf) -> Result<Self> {
    Self::new(path)
  }
}

impl TryFrom<&Path> for Page {
  type Error = crate::error::Error;

  fn try_from(path: &Path) -> Result<Self> {
    Self::new(path.to_path_buf())
  }
}

impl PartialOrd for Page {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for Page {
  fn cmp(&self, other: &Self) -> Ordering {
    natord::compare_ignore_case(&self.filename, &other.filename)
  }
}
