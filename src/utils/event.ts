import { router } from '@/router';
import { useReaderStore } from '@/stores';
import { convertBookSrc } from '@/utils/book';
import { listen } from '@tauri-apps/api/event';

enum AppEvent {
  BookOpened = 'book_opened'
}

export async function setupEventListeners() {
  await listen<Book>(AppEvent.BookOpened, (e) => {
    const store = useReaderStore();
    store.book = convertBookSrc(e.payload);
    void nextTick().then(() => router.push('/reader'));
  });
}
