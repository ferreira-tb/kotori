import { handleError } from 'manatsu';
import { useLibraryStore } from '../stores';
import { listen } from '@tauri-apps/api/event';

export enum Event {
  BookAdded = 'book_added',
  BookRemoved = 'book_removed',
  CoverExtracted = 'cover_extracted',
  LibraryCleared = 'library_cleared',
  RatingUpdated = 'rating_updated'
}

export function setupEventListeners() {
  const promises = Promise.all([
    onBookAdded(),
    onBookRemoved(),
    onCoverExtracted(),
    onLibraryCleared(),
    onRatingUpdated()
  ]);

  promises.catch(handleError);
}

function onBookAdded() {
  return listen<BookAddedPayload>(Event.BookAdded, ({ payload }) => {
    const store = useLibraryStore();
    store.library.add(payload);
  });
}

function onBookRemoved() {
  return listen<BookRemovedPayload>(Event.BookRemoved, ({ payload }) => {
    const store = useLibraryStore();
    store.library.remove(payload.id);
    if (store.selected?.id === payload.id) {
      store.selected = null;
    }
  });
}

function onCoverExtracted() {
  return listen<CoverExtractedPayload>(Event.CoverExtracted, ({ payload }) => {
    const store = useLibraryStore();
    store.library.setBookCover(payload.id, payload.path);
  });
}

function onLibraryCleared() {
  return listen(Event.LibraryCleared, () => {
    const store = useLibraryStore();
    store.library.load().catch(handleError);
    store.selected = null;
  });
}

function onRatingUpdated() {
  return listen<RatingUpdatedPayload>(Event.RatingUpdated, ({ payload }) => {
    const store = useLibraryStore();
    store.library.setBookRating(payload.id, payload.rating);
  });
}
