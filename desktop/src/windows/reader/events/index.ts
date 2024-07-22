import { handleError } from 'manatsu';
import { useReaderStore } from '../stores';
import { listen } from '@tauri-apps/api/event';

export const enum Event {
  PageDeleted = 'page_deleted',
  ReaderBookChanged = 'reader_book_changed'
}

export function setupEventListeners() {
  const promises = [onPageDeleted(), onReaderBookChanged()];
  Promise.all(promises).catch(handleError);
}

function onPageDeleted() {
  return listen<PageDeletedPayload>(Event.PageDeleted, ({ payload }) => {
    const store = useReaderStore();
    store.reader.removePage(payload.name);
  });
}

function onReaderBookChanged() {
  return listen(Event.ReaderBookChanged, () => {
    const store = useReaderStore();
    store.reader.load().catch(handleError);
  });
}
