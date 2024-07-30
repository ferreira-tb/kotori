mod actor;
mod message;
pub mod model;
mod schema;

use crate::book::Title;
use crate::database::model::prelude::*;
use crate::utils::path::PathExt;
use crate::utils::result::Result;
use actor::Actor;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use message::Message;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::{fs, thread};
use tauri::{AppHandle, Manager};

/// Send a message to the actor, awaiting its response with a oneshot channel.
macro_rules! send_tx {
  ($handle:expr, $message:ident { $($item:tt),* }) => {{
    let (tx, rx) = tokio::sync::oneshot::channel();
    let _ = $handle.sender.send(Message::$message { tx $(,$item)* });
    rx.await?
  }};
}

#[derive(Clone)]
pub struct DatabaseHandle {
  sender: mpsc::Sender<Message>,
}

impl DatabaseHandle {
  pub fn new(app: &AppHandle) -> Result<Self> {
    let path = app.path().app_local_data_dir()?;
    fs::create_dir_all(&path)?;

    #[cfg(not(any(debug_assertions, feature = "devtools")))]
    let path = path.join("kotori.db");
    #[cfg(any(debug_assertions, feature = "devtools"))]
    let path = path.join("kotori-dev.db");

    let database_url = path.try_str()?;
    let connection = SqliteConnection::establish(database_url)?;

    let (sender, receiver) = mpsc::channel();
    let mut actor = Actor::new(connection, receiver);
    
    thread::spawn(move || actor.run());

    Ok(Self { sender })
  }

  pub async fn get_all_books(&self) -> Result<Vec<Book>> {
    send_tx!(self, GetAllBooks {})
  }

  pub async fn get_all_collections(&self) -> Result<Vec<Collection>> {
    send_tx!(self, GetAllCollections {})
  }

  pub async fn get_all_folders(&self) -> Result<Vec<PathBuf>> {
    send_tx!(self, GetAllFolders {})
  }

  pub async fn get_book_by_id(&self, book_id: i32) -> Result<Book> {
    send_tx!(self, GetBookById { book_id })
  }

  pub async fn get_book_by_path(&self, book_path: impl AsRef<Path>) -> Result<Book> {
    let book_path = book_path.as_ref().to_owned();
    send_tx!(self, GetBookByPath { book_path })
  }

  pub async fn get_book_cover(&self, book_id: i32) -> Result<String> {
    send_tx!(self, GetBookCover { book_id })
  }

  pub async fn get_book_title(&self, book_id: i32) -> Result<Title> {
    send_tx!(self, GetBookTitle { book_id })
  }

  pub async fn random_book(&self) -> Result<Option<Book>> {
    send_tx!(self, RandomBook {})
  }

  #[cfg(any(debug_assertions, feature = "devtools"))]
  pub async fn remove_all_books(&self) -> Result<()> {
    send_tx!(self, RemoveAllBooks {})
  }

  #[cfg(any(debug_assertions, feature = "devtools"))]
  pub async fn remove_all_folders(&self) -> Result<()> {
    send_tx!(self, RemoveAllFolders {})
  }

  pub async fn remove_book(&self, book_id: i32) -> Result<()> {
    send_tx!(self, RemoveBook { book_id })
  }

  pub async fn save_book(&self, book: NewBook) -> Result<Book> {
    send_tx!(self, SaveBook { book })
  }

  pub async fn save_folders<I>(&self, folders: I) -> Result<()>
  where
    I: IntoIterator<Item = NewFolder>,
  {
    let folders = folders.into_iter().collect();
    send_tx!(self, SaveFolders { folders })
  }

  pub async fn update_book_cover(&self, book_id: i32, cover: &str) -> Result<Book> {
    let cover = cover.to_owned();
    send_tx!(self, UpdateBookCover { book_id, cover })
  }

  pub async fn update_book_rating(&self, book_id: i32, rating: u8) -> Result<Book> {
    send_tx!(self, UpdateBookRating { book_id, rating })
  }
}
