import { defineStore } from 'pinia';
import { Command } from '@/utils/commands';
import { useBook } from '../lib';

export const useReaderStore = defineStore('reader', () => {
  const { pages, ...book } = useBook();
  const windowId = useInvoke<number | null>(Command.GetCurrentReaderWindowId, null);

  function findNextIndex(current: number, step: number) {
    const index = pages.value.findIndex(({ id }) => id === current);
    return pages.value.at(index + step);
  }

  function lastIndex() {
    const index = pages.value.length - 1;
    return index >= 0 ? index : 0;
  }

  return {
    windowId: windowId.state,
    pages,
    current: book.current,
    findNextIndex,
    lastIndex,
    nextPage: book.next,
    previousPage: book.previous,
    firstPage: book.first,
    lastPage: book.last,
    removePage: book.remove
  };
});
