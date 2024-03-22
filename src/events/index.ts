import { router } from '@/router';
import { useReaderStore } from '@/stores';
import { convertBookSrc } from '@/utils/book';
import { listen } from '@tauri-apps/api/event';

enum AppEvent {
  BookOpened = 'book_opened',
  NavigateToLibrary = 'navigate_to_library'
}

export async function setupEventListeners() {
  await Promise.all([onBookOpened(), onNavigateToLibrary()]);
}

function onBookOpened() {
  return listen<Book>(AppEvent.BookOpened, (e) => {
    const store = useReaderStore();
    store.book = convertBookSrc(e.payload);
    void nextTick().then(() => router.push('/reader'));
  });
}

function onNavigateToLibrary() {
  return listen(AppEvent.NavigateToLibrary, () => {
    void router.push('/');
  });
}
