use crate::database::prelude::*;
use crate::prelude::*;
use crate::utils::glob;
use tauri_plugin_dialog::{DialogExt, FileDialogBuilder};
use walkdir::WalkDir;

pub struct Library;

impl Library {
  pub async fn from_dialog(app: &AppHandle) -> Result<()> {
    let (tx, rx) = oneshot::channel();
    let dialog = app.dialog().clone();

    FileDialogBuilder::new(dialog).pick_folders(move |response| {
      tx.send(response.unwrap_or_default()).ok();
    });

    let folders = rx.await?;
    if folders.is_empty() {
      return Ok(());
    }

    let globset = glob::book();
    let mut books = Vec::new();

    for folder in folders {
      for entry in WalkDir::new(&folder) {
        let path = entry?.into_path();
        if path.is_file() && globset.is_match(&path) {
          books.push(path);
        }
      }
    }

    Self::save_books(app, books).await?;

    Ok(())
  }

  pub async fn save_books(app: &AppHandle, paths: Vec<PathBuf>) -> Result<()> {
    let mut books = Vec::with_capacity(paths.len());
    for path in paths {
      let path = path
        .to_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| err!(InvalidBookPath, "{}", path.display()))?;

      let model = BookActiveModel {
        path: Set(path),
        ..Default::default()
      };

      books.push(model);
    }

    if books.is_empty() {
      return Ok(());
    }

    let on_conflict = OnConflict::column(BookColumn::Path)
      .do_nothing()
      .to_owned();

    let kotori = app.state::<Kotori>();
    Book::insert_many(books)
      .on_conflict(on_conflict)
      .do_nothing()
      .exec(&kotori.db)
      .await?;

    Ok(())
  }
}