use crate::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Title(pub(super) String);

impl Title {
  pub fn new(title: impl AsRef<str>) -> Self {
    Self(title.as_ref().to_owned())
  }

  pub fn as_str(&self) -> &str {
    self.0.as_str()
  }
}

impl TryFrom<&Path> for Title {
  type Error = crate::error::Error;

  fn try_from(path: &Path) -> Result<Self> {
    let title = path
      .file_stem()
      .ok_or_else(|| err!(InvalidPath, "{}", path.display()))?
      .to_string_lossy()
      .replace('_', " ");

    Ok(Self(title))
  }
}

impl TryFrom<&PathBuf> for Title {
  type Error = crate::error::Error;

  fn try_from(path: &PathBuf) -> Result<Self> {
    Title::try_from(path.as_path())
  }
}

impl TryFrom<&str> for Title {
  type Error = crate::error::Error;

  fn try_from(path: &str) -> Result<Self> {
    let path = Path::new(path);
    Title::try_from(path)
  }
}

impl fmt::Display for Title {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}
