use crate::prelude::*;
use crate::utils::glob;
use std::fs::File;
use tauri_plugin_dialog::{DialogExt, FileDialogBuilder};
use tokio::sync::oneshot;
use zip::ZipArchive;

#[derive(Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ActiveBook {
  pub path: PathBuf,
  pub title: String,

  #[serde(skip_serializing)]
  file: Option<BookFile>,
}

impl ActiveBook {
  pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
    let path = path.as_ref();
    let title = path
      .file_stem()
      .ok_or_else(|| err!(InvalidBook, "invalid book path: {path:?}"))?
      .to_string_lossy()
      .into_owned()
      .replace('_', " ");

    let book = Self {
      path: path.to_owned(),
      title,
      file: None,
    };

    Ok(book)
  }

  pub async fn from_dialog(app: &AppHandle) -> Result<Option<Self>> {
    let (tx, rx) = oneshot::channel();
    let dialog = app.dialog().clone();

    FileDialogBuilder::new(dialog)
      .add_filter("Book", &["cbr", "cbz"])
      .pick_file(move |response| {
        tx.send(response).ok();
      });

    if let Some(response) = rx.await? {
      let book = Self::new(response.path)?;
      return Ok(Some(book));
    }

    Ok(None)
  }
}

impl PartialEq for ActiveBook {
  fn eq(&self, other: &Self) -> bool {
    self.path == other.path
  }
}

impl Eq for ActiveBook {}

impl PartialOrd for ActiveBook {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for ActiveBook {
  fn cmp(&self, other: &Self) -> Ordering {
    natord::compare_ignore_case(&self.title, &other.title)
  }
}

#[derive(Debug)]
struct BookFile {
  handle: ZipArchive<File>,
  entries: Vec<String>,
}

impl BookFile {
  fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
    let zip = File::open(path.as_ref())?;
    let zip = ZipArchive::new(zip)?;

    let globset = glob::book_page();
    let entries = zip
      .file_names()
      .filter(|n| globset.is_match(n))
      .map_into::<String>()
      .collect_vec();

    if entries.is_empty() {
      bail!(Empty);
    }

    let file = Self {
      handle: zip,
      entries,
    };

    Ok(file)
  }
}
