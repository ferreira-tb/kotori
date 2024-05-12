mod payload;

use crate::book::LibraryBook;
use crate::prelude::*;
use crate::window::WindowKind;
use payload::{BookRemoved, CoverExtracted, RatingUpdated};
use serde::Serialize;
use strum::Display;

#[derive(Display)]
#[strum(serialize_all = "snake_case")]
pub enum Event<'a> {
  BookAdded(&'a LibraryBook),
  BookRemoved(i32),
  CoverExtracted { id: i32, path: &'a Path },
  PageDeleted { window_id: u16 },
  RatingUpdated { id: i32, rating: u8 },
}

impl<'a> Event<'a> {
  pub fn emit(self, app: &AppHandle) -> Result<()> {
    let event = self.to_string();

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
      Event::BookRemoved(id) => to_main!(BookRemoved::new(id)),
      Event::CoverExtracted { id, path } => to_main!(CoverExtracted::new(id, path)?),
      Event::PageDeleted { window_id, .. } => to_reader!(window_id, ()),
      Event::RatingUpdated { id, rating } => to_main!(RatingUpdated::new(id, rating)),
    }
  }
}

fn emit_to_main<S>(app: &AppHandle, event: &str, payload: S) -> Result<()>
where
  S: Serialize + Clone,
{
  debug!(event, target = "main");
  app
    .emit_to(WindowKind::Main, event, payload)
    .map_err(Into::into)
}

fn emit_to_reader<S>(app: &AppHandle, event: &str, id: u16, payload: S) -> Result<()>
where
  S: Serialize + Clone,
{
  debug!(event, target = "reader", id);
  app
    .emit_to(WindowKind::Reader(id), event, payload)
    .map_err(Into::into)
}
