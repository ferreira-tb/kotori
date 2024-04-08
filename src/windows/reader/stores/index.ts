import { defineStore } from 'pinia';
import { Command } from '@/utils/commands';
import { Page } from '../utils/page';

export const useReaderStore = defineStore('reader', () => {
  const book = useInvoke<ReaderBook | null>(Command.GetCurrentReaderBook, null);
  const readerId = useInvoke<number | null>(Command.GetCurrentReaderWindowId, null);
  watchEffect(() => console.log('Reader ID:', readerId.state.value));

  // This MUST be a ref.
  // Computed will fail to update once the page is fetched.
  const pages = ref<Page[]>([]);
  watchImmediate(book.state, (value) => {
    pages.value = value?.pages?.map((id) => new Page(id)) ?? [];
  });

  function findNext(current: number, step: number) {
    const index = pages.value.findIndex((page) => page.id === current);
    return pages.value.at(index + step);
  }

  function lastIndex() {
    const index = pages.value.length - 1;
    return index >= 0 ? index : 0;
  }

  return {
    readerId: readerId.state,
    book: book.state,
    pages,
    findNext,
    lastIndex
  };
});
