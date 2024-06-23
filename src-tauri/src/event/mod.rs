mod payload;

use crate::book::LibraryBook;
use crate::prelude::*;
use crate::window::WindowKind;
use payload::{BookRemoved, CoverExtracted, RatingUpdated};
use serde::Serialize;
use std::fmt;
use strum::{AsRefStr, Display};
use tauri::EventTarget;

#[allow(clippy::enum_variant_names)]
#[derive(AsRefStr, Clone, Display)]
#[strum(serialize_all = "snake_case")]
pub enum Event<'a> {
  BookAdded(&'a LibraryBook),
  BookRemoved(i32),
  ConfigUpdated(&'a str),
  CoverExtracted { id: i32, path: &'a Path },
  LibraryCleared,
  PageDeleted { window_id: u16 },
  RatingUpdated { id: i32, rating: u8 },
}

impl<'a> Event<'a> {
  pub fn emit(self, app: &AppHandle) -> Result<()> {
    let event = self.as_ref();

    macro_rules! to_main {
      ($payload:expr) => {{
        emit_to_main(app, &event, $payload)
      }};
    }

    macro_rules! to_reader {
      ($id:expr, $payload:expr) => {{
        emit_to_reader(app, &event, $id, $payload)
      }};
    }

    match self {
      Event::BookAdded(book) => to_main!(book),
      Event::BookRemoved(id) => to_main!(BookRemoved { id }),
      Event::ConfigUpdated(label) => emit_filter(app, &event, (), label),
      Event::CoverExtracted { id, path } => to_main!(CoverExtracted::new(id, path)?),
      Event::LibraryCleared => to_main!(()),
      Event::PageDeleted { window_id, .. } => to_reader!(window_id, ()),
      Event::RatingUpdated { id, rating } => to_main!(RatingUpdated { id, rating }),
    }
  }
}

fn emit_to_main<S>(app: &AppHandle, event: &str, payload: S) -> Result<()>
where
  S: Serialize + Clone + fmt::Debug,
{
  debug!(event, target = "main", ?payload);
  app
    .emit_to(WindowKind::Main, event, payload)
    .map_err(Into::into)
}

fn emit_to_reader<S>(app: &AppHandle, event: &str, id: u16, payload: S) -> Result<()>
where
  S: Serialize + Clone + fmt::Debug,
{
  debug!(event, target = "reader", id, ?payload);
  app
    .emit_to(WindowKind::Reader(id), event, payload)
    .map_err(Into::into)
}

fn emit_filter<S>(app: &AppHandle, event: &str, payload: S, exclude: &str) -> Result<()>
where
  S: Serialize + Clone + fmt::Debug,
{
  debug!(event, target = "all", exclude, ?payload);
  app.emit_filter(event, payload, |target| match target {
    EventTarget::WebviewWindow { label } => label != exclude,
    _ => false,
  })?;

  Ok(())
}
