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
  return listen<PageDeletedPayload>(Event.PageDeleted, ({ payload }) => {
    const store = useReaderStore();
    store.removePage(payload.page);
  });
}
