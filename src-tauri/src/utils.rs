use crate::error::Result;
use globset::{Glob, GlobBuilder};

pub fn img_glob(glob: &str) -> Result<Glob> {
  GlobBuilder::new(glob)
    .case_insensitive(true)
    .build()
    .map_err(Into::into)
}
