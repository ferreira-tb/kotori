pub mod book;
pub mod collection;
pub mod folder;

pub use book::{Book, NewBook};
pub use collection::Collection;
pub use folder::NewFolder;

pub mod prelude {
  pub use super::{Book, Collection, NewBook, NewFolder};
}
