use crate::prelude::*;
use crate::utils::glob;
use natord::compare_ignore_case;
use std::fs::File;
use std::io::Read;
use zip::ZipArchive;

pub struct ActiveBook {
  pub path: PathBuf,
  pub title: String,

  handle: ZipArchive<File>,
  pages: HashMap<usize, String>,
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

  pub async fn from_dialog(app: &AppHandle) -> Result<Vec<Self>> {
    let (tx, rx) = oneshot::channel();
    let dialog = app.dialog().clone();

    FileDialogBuilder::new(dialog)
      .add_filter("Book", &["cbr", "cbz"])
      .pick_files(move |response| {
        tx.send(response).ok();
      });

    if let Some(response) = rx.await? {
      return response
        .into_iter()
        .map(|r| Self::new(r.path))
        .collect();
    }

    Ok(Vec::new())
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
