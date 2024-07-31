use crate::book::Title;
use crate::database::model::prelude::*;
use crate::utils::result::TxResult;
use std::fmt;
use std::path::PathBuf;
use strum::Display;

#[derive(Display)]
#[strum(serialize_all = "snake_case")]
pub enum Message {
  GetAllBooks {
    tx: TxResult<Vec<Book>>,
  },
  GetAllCollections {
    tx: TxResult<Vec<Collection>>,
  },
  GetAllFolders {
    tx: TxResult<Vec<PathBuf>>,
  },
  GetBookById {
    book_id: i32,
    tx: TxResult<Book>,
  },
  GetBookByPath {
    book_path: PathBuf,
    tx: TxResult<Book>,
  },
  GetBookCover {
    book_id: i32,
    tx: TxResult<String>,
  },
  GetBookTitle {
    book_id: i32,
    tx: TxResult<Title>,
  },
  RandomBook {
    tx: TxResult<Option<Book>>,
  },
  RemoveBook {
    book_id: i32,
    tx: TxResult<()>,
  },
  SaveBook {
    book: NewBook,
    tx: TxResult<Book>,
  },
  SaveFolders {
    folders: Vec<NewFolder>,
    tx: TxResult<()>,
  },
  UpdateBookCover {
    book_id: i32,
    cover: String,
    tx: TxResult<Book>,
  },
  UpdateBookRating {
    book_id: i32,
    rating: u8,
    tx: TxResult<Book>,
  },

  #[cfg(feature = "devtools")]
  RemoveAllBooks {
    tx: TxResult<()>,
  },
  #[cfg(feature = "devtools")]
  RemoveAllFolders {
    tx: TxResult<()>,
  },
}

impl fmt::Debug for Message {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_tuple("Message")
      .field(&self.to_string())
      .finish()
  }
}
