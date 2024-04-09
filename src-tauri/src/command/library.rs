use crate::book::{ActiveBook, Cover, IntoJson, LibraryBook};
use crate::database::prelude::*;
use crate::event::Event;
use crate::prelude::*;

#[tauri::command]
pub async fn add_to_library_from_dialog(app: AppHandle) -> Result<()> {
  let kotori = app.state::<Kotori>();
  kotori.library.add_from_dialog().await
}

#[tauri::command]
pub async fn get_library_books(app: AppHandle) -> Result<Json> {
  let kotori = app.state::<Kotori>();
  let books = Book::find().all(&kotori.db).await?;

  let tasks = books.into_iter().map(|model| {
    let app = app.clone();
    async_runtime::spawn(async move {
      let value = LibraryBook(&app, &model).into_json().await;
      if matches!(value, Ok(ref it) if it.get("cover").is_some_and(Json::is_null)) {
        let Ok(book) = ActiveBook::with_model(&model) else {
          return value.ok();
        };

        if let Ok(cover) = Cover::path(&app, model.id) {
          book.extract_cover(&app, cover);
        }
      }

      value.ok()
    })
  });

  let books = join_all(tasks)
    .await
    .into_iter()
    .filter_map(std::result::Result::unwrap_or_default)
    .collect_vec();

  Ok(Json::Array(books))
}

#[tauri::command]
pub async fn update_book_rating(app: AppHandle, id: i32, rating: u8) -> Result<()> {
  if rating > 5 {
    return Err(err!(InvalidRating));
  }

  let kotori = app.state::<Kotori>();
  let book = Book::find_by_id(id)
    .one(&kotori.db)
    .await?
    .ok_or_else(|| err!(BookNotFound))?;

  let mut book: BookActiveModel = book.into();
  book.rating = Set(i32::from(rating));
  book.update(&kotori.db).await?;

  let event = Event::RatingUpdated { id, rating };
  event.emit(&app)
}
