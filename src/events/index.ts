import { router } from '@/router';
import { convertBookSrc } from '@/utils';
import { useReaderStore } from '@/stores';
import { listen } from '@tauri-apps/api/event';

enum Event {
  AddToLibrary = 'add_to_library',
  NavigateToLibrary = 'navigate_to_library',
  OpenBook = 'open_book'
}

export function setupEventListeners() {
  return Promise.all([onBookOpened(), onNavigateToLibrary()]);
}

function onBookOpened() {
  return listen<Book>(Event.OpenBook, (e) => {
    const store = useReaderStore();
    store.book = convertBookSrc(e.payload);
    void nextTick().then(() => router.push('/reader'));
  });
}

function onNavigateToLibrary() {
  return listen(Event.NavigateToLibrary, () => {
    void router.push('/');
  });
}
