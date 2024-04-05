use crate::prelude::*;
use crate::utils::glob;
use natord::compare_ignore_case;
use serde::Serialize;
use std::cmp::Ordering;
use std::fmt;
use std::fs::File;
use std::io::Read;
use tauri_plugin_dialog::{DialogExt, FileDialogBuilder};
use zip::ZipArchive;

pub struct ReaderBook {
  pub path: PathBuf,
  pub title: Title,

  handle: ZipArchive<File>,
  pages: HashMap<usize, String>,
}

impl ReaderBook {
  pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
    let path = path.as_ref();
    let title = path.try_into()?;

    let file = File::open(path)?;
    let handle = ZipArchive::new(file)?;

    let globset = glob::book_page();
    let pages = handle
      .file_names()
      .filter(|n| globset.is_match(n))
      .sorted_unstable_by(|a, b| compare_ignore_case(a, b))
      .enumerate()
      .map(|(i, p)| (i, p.to_string()))
      .collect();

    let book = Self {
      path: path.to_owned(),
      title,
      handle,
      pages,
    };

    Ok(book)
  }

  async fn show_dialog(app: &AppHandle) -> Result<Vec<Self>> {
    let (tx, rx) = oneshot::channel();
    let dialog = app.dialog().clone();

    FileDialogBuilder::new(dialog)
      .add_filter("Book", &["cbr", "cbz"])
      .pick_files(move |response| {
        tx.send(response).ok();
      });

    if let Some(response) = rx.await? {
      return response
        .into_par_iter()
        .map(|r| Self::new(r.path))
        .collect();
    }

    Ok(Vec::new())
  }

  pub async fn open_book_from_dialog(app: &AppHandle) -> Result<()> {
    let books = Self::show_dialog(app).await?;

    if !books.is_empty() {
      let kotori = app.state::<Kotori>();
      let mut reader = kotori.reader.write().await;
      return reader.open_many(books).await.map_err(Into::into);
    }

    Ok(())
  }

  pub fn as_value(&self) -> Value {
    let pages = self
      .pages
      .keys()
      .copied()
      .sorted_unstable()
      .collect_vec();

    json!({
      "path": self.path,
      "title": self.title,
      "pages": pages
    })
  }

  pub fn get_cover_as_bytes(&mut self) -> Result<Vec<u8>> {
    self.get_page_as_bytes(0)
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

impl PartialEq for ReaderBook {
  fn eq(&self, other: &Self) -> bool {
    self.path == other.path
  }
}

impl Eq for ReaderBook {}

impl PartialOrd for ReaderBook {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for ReaderBook {
  fn cmp(&self, other: &Self) -> Ordering {
    compare_ignore_case(&self.title.0, &other.title.0)
  }
}

#[derive(Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Title(String);

impl TryFrom<&Path> for Title {
  type Error = crate::error::Error;

  fn try_from(path: &Path) -> Result<Self> {
    let title = path
      .file_stem()
      .ok_or_else(|| err!(InvalidBook, "invalid book path: {path:?}"))?
      .to_string_lossy()
      .replace('_', " ");

    Ok(Self(title))
  }
}

impl fmt::Display for Title {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}
