import { handleError } from 'manatsu';
import { listen } from '@tauri-apps/api/event';
import { useLibraryStore } from '../stores';

enum Event {
  BookAdded = 'book_added',
  CoverExtracted = 'cover_extracted',
  RatingUpdated = 'rating_updated'
}

export function setupEventListeners() {
  const promises = Promise.all([onBookAdded(), onCoverExtracted(), onRatingUpdated()]);
  promises.catch(handleError);
}

function onBookAdded() {
  return listen<LibraryBook>(Event.BookAdded, ({ payload }) => {
    const store = useLibraryStore();
    store.addBook(payload);
  });
}

function onCoverExtracted() {
  return listen<CoverExtractedPayload>(Event.CoverExtracted, ({ payload }) => {
    const store = useLibraryStore();
    store.updateBookCover(payload.id, payload.path);
  });
}

function onRatingUpdated() {
  return listen<RatingUpdatedPayload>(Event.RatingUpdated, ({ payload }) => {
    const store = useLibraryStore();
    store.updateBookRating(payload.id, payload.rating);
  });
}
