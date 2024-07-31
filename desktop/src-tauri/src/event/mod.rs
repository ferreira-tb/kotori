mod payload;

use crate::book::LibraryBook;
use crate::prelude::*;
use crate::window::WindowKind;
use payload::{BookRemoved, CoverExtracted, PageDeleted, RatingUpdated};
use serde::Serialize;
use std::fmt;
use strum::{AsRefStr, Display};
use tauri::Emitter;

#[allow(clippy::enum_variant_names)]
#[derive(AsRefStr, Clone, Debug, Display)]
#[strum(serialize_all = "snake_case")]
pub enum Event<'a> {
  BookAdded(&'a LibraryBook),
  BookRemoved(i32),
  CoverExtracted {
    id: i32,
    path: &'a Path,
  },
  PageDeleted {
    window_id: u16,
    name: &'a str,
  },
  RatingUpdated {
    id: i32,
    rating: u8,
  },
  ReaderBookChanged {
    window_id: u16,
  },

  #[cfg(feature = "devtools")]
  LibraryCleared,
}

impl<'a> Event<'a> {
  #[cfg_attr(feature = "tracing", instrument(skip(app)))]
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
      Event::CoverExtracted { id, path } => to_main!(CoverExtracted::new(id, path)?),
      Event::PageDeleted { window_id, name } => to_reader!(window_id, PageDeleted::new(name)),
      Event::RatingUpdated { id, rating } => to_main!(RatingUpdated { id, rating }),
      Event::ReaderBookChanged { window_id } => to_reader!(window_id, ()),

      #[cfg(feature = "devtools")]
      Event::LibraryCleared => to_main!(()),
    }
  }
}

fn emit_to_main<P>(app: &AppHandle, event: &str, payload: P) -> Result<()>
where
  P: Serialize + Clone + fmt::Debug,
{
  #[cfg(feature = "tracing")]
  debug!(event, target = "main", ?payload);

  app
    .emit_to(WindowKind::Main, event, payload)
    .map_err(Into::into)
}

fn emit_to_reader<P>(app: &AppHandle, event: &str, id: u16, payload: P) -> Result<()>
where
  P: Serialize + Clone + fmt::Debug,
{
  #[cfg(feature = "tracing")]
  debug!(event, target = "reader", id, ?payload);

  app
    .emit_to(WindowKind::Reader(id), event, payload)
    .map_err(Into::into)
}
