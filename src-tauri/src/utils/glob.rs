use globset::{Glob, GlobBuilder, GlobSet, GlobSetBuilder};

fn glob(glob: &str) -> Glob {
  GlobBuilder::new(glob)
    .case_insensitive(true)
    .build()
    .unwrap()
}

pub fn book() -> GlobSet {
  GlobSetBuilder::new()
    .add(glob("*.cbr"))
    .add(glob("*.cbz"))
    .build()
    .unwrap()
}

pub fn book_page() -> GlobSet {
  GlobSetBuilder::new()
    .add(glob("*.gif"))
    .add(glob("*.jpg"))
    .add(glob("*.jpeg"))
    .add(glob("*.png"))
    .add(glob("*.webp"))
    .build()
    .unwrap()
}
