mod payload;

use crate::book::LibraryBook;
use crate::prelude::*;
use crate::window::WindowKind;
use payload::{BookRemoved, CoverExtracted, PageDeleted, RatingUpdated};
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
  PageDeleted { window_id: u16, name: &'a str },
  RatingUpdated { id: i32, rating: u8 },
}

impl<'a> Event<'a> {
  pub fn emit(self, app: &AppHandle) -> Result<()> {
    let event = self.as_ref();

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
      Event::ConfigUpdated(label) => emit_filter(app, event, (), label),
      Event::CoverExtracted { id, path } => to_main!(CoverExtracted::new(id, path)?),
      Event::LibraryCleared => to_main!(()),
      Event::PageDeleted { window_id, name } => to_reader!(window_id, PageDeleted::new(name)),
      Event::RatingUpdated { id, rating } => to_main!(RatingUpdated { id, rating }),
    }
  }
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
