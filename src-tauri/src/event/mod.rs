mod payload;

use std::fmt;

use payload::{BookRemoved, CoverExtracted, PageDeleted, RatingUpdated};
use serde::Serialize;
use strum::{AsRefStr, Display};
use tauri::EventTarget;

use crate::{book::LibraryBook, prelude::*, window::WindowKind};

#[allow(clippy::enum_variant_names)]
#[derive(AsRefStr, Clone, Display)]
#[strum(serialize_all = "snake_case")]
pub enum Event<'a> {
  BookAdded(&'a LibraryBook),
  BookRemoved(i32),
  ConfigUpdated(Option<&'a str>),
  CoverExtracted { id: i32, path: &'a Path },
  LibraryCleared,
  PageDeleted { window_id: u16, name: &'a str },
  RatingUpdated { id: i32, rating: u8 },
}

impl<'a> Event<'a> {
  pub fn emit(self, app: &AppHandle) -> Result<()> {
    let event = self.as_ref();

    macro_rules! to_all {
      ($payload:expr) => {
        emit_all(app, &event, $payload)
      };
    }

    macro_rules! to_filter {
      ($payload:expr, $($exclude:expr)*) => {{
        $(emit_filter(app, event, $payload, $exclude)?;)*
        Ok(())
      }};
    }

    macro_rules! to_main {
      ($payload:expr) => {{
        emit_to_main(app, event, $payload)
      }};
    }

    macro_rules! to_reader {
      ($id:expr, $payload:expr) => {{
        emit_to_reader(app, event, $id, $payload)
      }};
    }

    match self {
      Event::BookAdded(book) => to_main!(book),
      Event::BookRemoved(id) => to_main!(BookRemoved { id }),
      Event::ConfigUpdated(label) => match label {
        Some(label) => to_filter!((), label),
        None => to_all!(()),
      },
      Event::CoverExtracted { id, path } => to_main!(CoverExtracted::new(id, path)?),
      Event::LibraryCleared => to_main!(()),
      Event::PageDeleted { window_id, name } => to_reader!(window_id, PageDeleted::new(name)),
      Event::RatingUpdated { id, rating } => to_main!(RatingUpdated { id, rating }),
    }
  }
}

fn emit_all<S>(app: &AppHandle, event: &str, payload: S) -> Result<()>
where
  S: Serialize + Clone + fmt::Debug,
{
  debug!(event, target = "all", ?payload);
  app.emit_filter(event, payload, |target| {
    matches!(target, EventTarget::WebviewWindow { .. })
  })?;

  Ok(())
}

fn emit_filter<P>(app: &AppHandle, event: &str, payload: P, exclude: &str) -> Result<()>
where
  P: Serialize + Clone + fmt::Debug,
{
  debug!(event, target = "all", exclude, ?payload);
  app.emit_filter(event, payload, |target| match target {
    EventTarget::WebviewWindow { label } => label != exclude,
    _ => false,
  })?;

  Ok(())
}

fn emit_to_main<P>(app: &AppHandle, event: &str, payload: P) -> Result<()>
where
  P: Serialize + Clone + fmt::Debug,
{
  debug!(event, target = "main", ?payload);
  app
    .emit_to(WindowKind::Main, event, payload)
    .map_err(Into::into)
}

fn emit_to_reader<P>(app: &AppHandle, event: &str, id: u16, payload: P) -> Result<()>
where
  P: Serialize + Clone + fmt::Debug,
{
  debug!(event, target = "reader", id, ?payload);
  app
    .emit_to(WindowKind::Reader(id), event, payload)
    .map_err(Into::into)
}
