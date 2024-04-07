use crate::prelude::*;
use crate::utils::event::Event;

pub enum Cover {
  Extracted(PathBuf),
  NotExtracted,
}

impl Cover {
  pub fn path(app: &AppHandle, book_id: i32) -> Result<PathBuf> {
    app
      .path()
      .app_cache_dir()
      .map(|dir| dir.join(format!("covers/{book_id}")))
      .map_err(Into::into)
  }

  pub fn notify(self, app: &AppHandle, book_id: i32) -> Result<()> {
    if let Self::Extracted(path) = self {
      let event = Event::CoverExtracted;
      let payload = json!({
        "id": book_id,
        "cover": path,
      });

      return app
        .emit_to(Event::target(), event.as_ref(), payload)
        .map_err(Into::into);
    };

    Ok(())
  }
}

impl From<PathBuf> for Cover {
  fn from(path: PathBuf) -> Self {
    Self::Extracted(path)
  }
}

impl From<Cover> for Value {
  fn from(cover: Cover) -> Self {
    match cover {
      Cover::Extracted(path) => json!(path),
      Cover::NotExtracted => Value::Null,
    }
  }
}
