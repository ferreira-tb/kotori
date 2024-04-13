use crate::prelude::*;
use serde::Serialize;
use std::fmt;

#[derive(Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Title(pub(super) String);

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
