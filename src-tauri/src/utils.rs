use crate::error::Result;
use globset::{Glob, GlobBuilder, GlobSet, GlobSetBuilder};

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
