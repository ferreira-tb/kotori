import { handleError } from 'manatsu';
import { listen } from '@tauri-apps/api/event';

export enum Event {
  PageDeleted = 'page_deleted'
}

export function setupEventListeners() {
  onPageDeleted().catch(handleError);
}

function onPageDeleted() {
  return listen(Event.PageDeleted, () => {
    unimplemented();
  });
}
