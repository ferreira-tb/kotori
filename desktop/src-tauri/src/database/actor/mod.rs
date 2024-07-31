mod book;
mod collection;
mod folder;

use crate::database::message::Message;
use diesel::sqlite::SqliteConnection;
use std::sync::mpsc;

type Db<'a> = &'a mut SqliteConnection;

macro_rules! send {
  ($tx:expr, $result:expr) => {
    let _ = $tx.send($result);
  };
}

pub(super) struct Actor {
  db: SqliteConnection,
  receiver: mpsc::Receiver<Message>,
}

impl Actor {
  pub(super) fn new(db: SqliteConnection, receiver: mpsc::Receiver<Message>) -> Self {
    Self { db, receiver }
  }

  pub(super) fn run(&mut self) {
    while let Ok(message) = self.receiver.recv() {
      self.handle_message(message);
    }
  }

  fn handle_message(&mut self, message: Message) {
    #[cfg(feature = "tracing")]
    tracing::trace!(%message);

    match message {
      Message::GetAllBooks { tx } => {
        send!(tx, book::get_all(&mut self.db));
      }
      Message::GetAllCollections { tx } => {
        send!(tx, collection::get_all(&mut self.db));
      }
      Message::GetAllFolders { tx } => {
        send!(tx, folder::get_all(&mut self.db));
      }
      Message::GetBookById { book_id, tx } => {
        send!(tx, book::get_by_id(&mut self.db, book_id));
      }
      Message::GetBookByPath { book_path, tx } => {
        send!(tx, book::get_by_path(&mut self.db, &book_path));
      }
      Message::GetBookCover { book_id, tx } => {
        send!(tx, book::get_cover(&mut self.db, book_id));
      }
      Message::GetBookPath { book_id, tx } => {
        send!(tx, book::get_path(&mut self.db, book_id));
      }
      Message::GetBookTitle { book_id, tx } => {
        send!(tx, book::get_title(&mut self.db, book_id));
      }
      Message::HasAnyBook { tx } => {
        send!(tx, book::is_empty(&mut self.db));
      }
      Message::HasAnyFolder { tx } => {
        send!(tx, folder::is_empty(&mut self.db));
      }
      Message::RandomBook { tx } => {
        send!(tx, book::random(&mut self.db));
      }
      Message::RemoveBook { book_id, tx } => {
        send!(tx, book::remove(&mut self.db, book_id));
      }
      Message::SaveBook { book, tx } => {
        send!(tx, book::save(&mut self.db, &book));
      }
      Message::SaveFolders { folders, tx } => {
        send!(tx, folder::save_many(&mut self.db, &folders));
      }
      Message::UpdateBookCover { book_id, cover, tx } => {
        send!(tx, book::update_cover(&mut self.db, book_id, &cover));
      }
      Message::UpdateBookRating { book_id, rating, tx } => {
        send!(tx, book::update_rating(&mut self.db, book_id, rating));
      }
      Message::UpdateBookRead { book_id, read, tx } => {
        send!(tx, book::update_read(&mut self.db, book_id, read));
      }

      #[cfg(feature = "devtools")]
      Message::RemoveAllBooks { tx } => {
        send!(tx, book::remove_all(&mut self.db));
      }
      #[cfg(feature = "devtools")]
      Message::RemoveAllFolders { tx } => {
        send!(tx, folder::remove_all(&mut self.db));
      }
    }
  }
}
