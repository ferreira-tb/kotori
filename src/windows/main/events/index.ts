import { handleError } from 'manatsu';
import { listen } from '@tauri-apps/api/event';
import { RouteName, router } from '../router';

enum Event {
  NavigateToLibrary = 'navigate_to_library'
}

export function setupEventListeners() {
  return Promise.all([onNavigateToLibrary()]);
}

function onNavigateToLibrary() {
  return listen(Event.NavigateToLibrary, () => {
    router.push({ name: RouteName.Library }).catch(handleError);
  });
}
