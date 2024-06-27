import { handleError } from 'manatsu';
import { useReaderStore } from '../stores';
import { listen } from '@tauri-apps/api/event';

export enum Event {
  PageDeleted = 'page_deleted'
}

export function setupEventListeners() {
  onPageDeleted().catch(handleError);
}

function onPageDeleted() {
  return listen<PageDeletedPayload>(Event.PageDeleted, ({ payload }) => {
    const store = useReaderStore();
    store.reader.removePage(payload.name);
  });
}
