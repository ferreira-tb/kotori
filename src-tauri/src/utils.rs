pub mod app {
  use crate::error::Result;
  use crate::{err, Kotori};
  use tauri::{Manager, Runtime, State, WebviewWindow};

  pub trait AppHandleExt<R: Runtime>: Manager<R> {
    fn kotori(&self) -> State<Kotori> {
      self.state::<Kotori>()
    }

    fn get_main_window(&self) -> Result<WebviewWindow<R>> {
      self
        .get_webview_window("main")
        .ok_or_else(|| err!(WindowNotFound, "main"))
    }
  }

  impl<R: Runtime> AppHandleExt<R> for tauri::AppHandle<R> {}
}

pub mod collections {
  use ahash::AHasher;
  use indexmap::IndexMap;
  use std::hash::BuildHasherDefault;

  pub type OrderedMap<K, V> = IndexMap<K, V, BuildHasherDefault<AHasher>>;
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

pub mod path {
  use crate::prelude::*;

  pub fn parent(path: &Path) -> Result<&Path> {
    path
      .parent()
      .ok_or_else(|| err!(InvalidPath, "{}", path.display()))
  }

  pub fn to_str(path: &Path) -> Result<&str> {
    path
      .to_str()
      .ok_or_else(|| err!(InvalidPath, "{}", path.display()))
  }

  pub fn to_string(path: impl AsRef<Path>) -> Result<String> {
    to_str(path.as_ref()).map(ToOwned::to_owned)
  }
}

pub mod window {
  use crate::prelude::*;
  use tauri::WebviewUrl;

  pub fn data_directory(app: &AppHandle, name: impl AsRef<str>) -> Result<PathBuf> {
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
