use crate::error::{Error, Result};
use std::cmp::Ordering;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
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
