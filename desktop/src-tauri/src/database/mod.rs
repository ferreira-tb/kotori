mod actor;
mod message;
pub mod model;
mod schema;

use crate::book::{ActiveBook, Title};
use crate::database::model::prelude::*;
use crate::event::Event;
use crate::menu::AppMenu;
use crate::path::PathExt;
use crate::result::Result;
use crate::send_tx;
use actor::Actor;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use message::Message;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::{fs, thread};
use tauri::{AppHandle, Manager};

const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

#[derive(Clone)]
pub struct DatabaseHandle {
  app: AppHandle,
  sender: mpsc::Sender<Message>,
}

impl DatabaseHandle {
  pub fn new(app: &AppHandle) -> Result<Self> {
    let path = app.path().app_local_data_dir()?;
    fs::create_dir_all(&path)?;

    #[cfg(feature = "devtools")]
    let path = path.join("kotori-dev.db");
    #[cfg(not(feature = "devtools"))]
    let path = path.join("kotori.db");

    let database_url = path.try_str()?;
    let mut connection = SqliteConnection::establish(database_url)?;
    connection
      .run_pending_migrations(MIGRATIONS)
      .unwrap();

    let (sender, receiver) = mpsc::channel();
    let mut actor = Actor::new(connection, receiver);

    let app = app.clone();
    app.run_on_main_thread(move || {
      thread::Builder::new()
        .name("database-worker".into())
        .spawn(move || actor.run())
        .expect("failed to spawn database worker");
    })?;

    Ok(Self { app, sender })
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

  pub async fn get_book_path(&self, book_id: i32) -> Result<PathBuf> {
    send_tx!(self, GetBookPath { book_id })
  }

  pub async fn get_book_title(&self, book_id: i32) -> Result<Title> {
    send_tx!(self, GetBookTitle { book_id })
  }

  pub async fn has_any_book(&self) -> Result<bool> {
    send_tx!(self, HasAnyBook {})
  }

  pub async fn has_any_folder(&self) -> Result<bool> {
    send_tx!(self, HasAnyFolder {})
  }

  pub async fn has_book_path(&self, book_path: &Path) -> Result<bool> {
    let book_path = book_path.to_owned();
    send_tx!(self, HasBookPath { book_path })
  }

  pub async fn random_book(&self) -> Result<Option<Book>> {
    send_tx!(self, RandomBook {})
  }

  #[cfg(feature = "devtools")]
  pub async fn remove_all_books(&self) -> Result<()> {
    send_tx!(self, RemoveAllBooks {})?;
    AppMenu::spawn_update(&self.app);
    Ok(())
  }

  #[cfg(feature = "devtools")]
  pub async fn remove_all_folders(&self) -> Result<()> {
    send_tx!(self, RemoveAllFolders {})
  }

  pub async fn remove_book(&self, book_id: i32) -> Result<()> {
    send_tx!(self, RemoveBook { book_id })?;
    AppMenu::spawn_update(&self.app);
    Ok(())
  }

  pub async fn save_book(&self, book: NewBook) -> Result<Book> {
    let book = send_tx!(self, SaveBook { book })?;
    AppMenu::spawn_update(&self.app);
    Ok(book)
  }

  pub async fn save_folders<I>(&self, folders: I) -> Result<()>
  where
    I: IntoIterator<Item = NewFolder>,
  {
    let folders = folders.into_iter().collect();
    send_tx!(self, SaveFolders { folders })?;
    AppMenu::spawn_update(&self.app);
    Ok(())
  }

  /// Set the specified page as the book cover, extracting it afterwards.
  pub async fn update_book_cover(&self, book_id: i32, cover: &str) -> Result<Book> {
    let cover = cover.to_owned();
    let book = send_tx!(self, UpdateBookCover { book_id, cover })?;

    let active = ActiveBook::from_model(&self.app, &book)?;
    active.extract_cover().await?;
    book.save_as_metadata(&self.app).await?;

    Ok(book)
  }

  pub async fn update_book_rating(&self, book_id: i32, rating: u8) -> Result<Book> {
    let book = send_tx!(self, UpdateBookRating { book_id, rating })?;

    Event::RatingUpdated { id: book_id, rating }.emit(&self.app)?;
    book.save_as_metadata(&self.app).await?;

    Ok(book)
  }

  pub async fn update_book_read(&self, book_id: i32, read: bool) -> Result<Book> {
    let book = send_tx!(self, UpdateBookRead { book_id, read })?;

    Event::ReadUpdated { id: book_id, read }.emit(&self.app)?;
    book.save_as_metadata(&self.app).await?;

    Ok(book)
  }
}
