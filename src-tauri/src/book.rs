use crate::prelude::*;
use crate::utils::glob;
use natord::compare_ignore_case;
use std::fs::File;
use std::io::Read;
use tauri_plugin_dialog::{DialogExt, FileDialogBuilder};
use tokio::sync::oneshot;
use zip::ZipArchive;

pub struct ActiveBook {
  pub file: BookFile,
  pub path: PathBuf,
  pub title: String,
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
      file: BookFile::new(path)?,
      path: path.to_owned(),
      title,
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
    compare_ignore_case(&self.title, &other.title)
  }
}

pub struct BookFile {
  handle: ZipArchive<File>,
  pub pages: HashMap<usize, String>,
}

impl BookFile {
  fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
    let zip = File::open(path.as_ref())?;
    let zip = ZipArchive::new(zip)?;

    let globset = glob::book_page();
    let pages: HashMap<usize, String> = zip
      .file_names()
      .filter(|n| globset.is_match(n))
      .sorted_unstable_by(|a, b| compare_ignore_case(a, b))
      .enumerate()
      .map(|(i, p)| (i, p.to_string()))
      .collect();

    if pages.is_empty() {
      bail!(Empty);
    }

    let file = Self { handle: zip, pages };

    Ok(file)
  }

  pub fn get_page_as_bytes(&mut self, page: usize) -> Result<Vec<u8>> {
    let name = self
      .pages
      .get(&page)
      .ok_or_else(|| err!(PageNotFound))?;

    let mut file = self.handle.by_name(name)?;
    let size = usize::try_from(file.size()).unwrap_or_default();
    let mut buf = Vec::with_capacity(size);
    file.read_to_end(&mut buf)?;

    Ok(buf)
  }
}
