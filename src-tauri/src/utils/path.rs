use crate::err;
use crate::error::Result;
use std::path::Path;

pub trait PathExt {
  fn try_parent(&self) -> Result<&Path>;
  fn try_to_str(&self) -> Result<&str>;
  fn try_to_string(&self) -> Result<String>;
}

impl<P: AsRef<Path>> PathExt for P {
  fn try_parent(&self) -> Result<&Path> {
    let path = self.as_ref();
    path
      .parent()
      .ok_or_else(|| err!(InvalidPath, "{}", path.display()))
  }

  fn try_to_str(&self) -> Result<&str> {
    let path = self.as_ref();
    path
      .to_str()
      .ok_or_else(|| err!(InvalidPath, "{}", path.display()))
  }

  fn try_to_string(&self) -> Result<String> {
    self.try_to_str().map(ToOwned::to_owned)
  }
}
