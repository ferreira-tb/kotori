use ahash::AHasher;
use indexmap::IndexMap;
use std::hash::BuildHasherDefault;

pub type OrderedMap<K, V> = IndexMap<K, V, BuildHasherDefault<AHasher>>;

pub mod event {
  use strum::{AsRefStr, Display, EnumString};
  use tauri::EventTarget;

  #[derive(AsRefStr, Display, EnumString)]
  #[strum(serialize_all = "snake_case")]
  pub enum Event {
    BookAdded,
    CoverExtracted,
  }

  impl Event {
    pub fn target() -> EventTarget {
      EventTarget::WebviewWindow {
        label: "main".into(),
      }
    }
  }
}

pub mod glob {
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
      .add(glob("*.bmp"))
      .add(glob("*.gif"))
      .add(glob("*.jpg"))
      .add(glob("*.jpeg"))
      .add(glob("*.png"))
      .add(glob("*.webp"))
      .build()
      .unwrap()
  }
}

pub mod window {
  use crate::prelude::*;
  use tauri::WebviewUrl;

  pub fn dir(app: &AppHandle, name: impl AsRef<str>) -> Result<PathBuf> {
    let name = name.as_ref();
    let path = app
      .path()
      .app_data_dir()?
      .join(format!("windows/{name}"));

    Ok(path)
  }

  pub fn webview_url(name: impl AsRef<str>) -> WebviewUrl {
    let name = name.as_ref();
    WebviewUrl::App(format!("src/windows/{name}/index.html",).into())
  }
}
