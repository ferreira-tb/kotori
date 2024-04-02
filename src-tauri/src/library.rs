use crate::database::prelude::Book;

#[derive(Default)]
pub struct Library {
  books: Vec<Book>,
}

impl Library {
  pub fn new() -> Self {
    Self { books: Vec::new() }
  }
}
