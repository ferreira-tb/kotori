import { handleError } from 'manatsu';
import { listen } from '@tauri-apps/api/event';
import { useLibraryStore } from '../stores';

export enum Event {
  BookAdded = 'book_added',
  BookRemoved = 'book_removed',
  CoverExtracted = 'cover_extracted',
  RatingUpdated = 'rating_updated',
  RemoveBookRequested = 'remove_book_requested'
}

export function setupEventListeners() {
  const promises = Promise.all([
    onBookAdded(),
    onBookRemoved(),
    onCoverExtracted(),
    onRatingUpdated()
  ]);

  promises.catch(handleError);
}

function onBookAdded() {
  return listen<BookAddedPayload>(Event.BookAdded, ({ payload }) => {
    const store = useLibraryStore();
    store.addBook(payload);
  });
}

function onBookRemoved() {
  return listen<BookRemovedPayload>(Event.BookRemoved, ({ payload }) => {
    const store = useLibraryStore();
    store.removeBook(payload.id);
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
