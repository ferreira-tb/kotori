import { handleError } from 'manatsu';
import { listen } from '@tauri-apps/api/event';
import { useReaderStore } from '../stores';

export enum Event {
  DeletePageRequested = 'delete_page_requested',
  PageDeleted = 'page_deleted'
}

export function setupEventListeners() {
  onPageDeleted().catch(handleError);
}

function onPageDeleted() {
  return listen(Event.PageDeleted, () => {
    // When a page is deleted, the index of the other pages may change.
    // Reloading the book is the safest way to ensure consistency.
    //
    // This is too aggressive though, as we lose the eagerly fetched pages.
    // Future me, please find a better way to handle this.
    const store = useReaderStore();
    store.reload().catch(handleError);
  });
}
