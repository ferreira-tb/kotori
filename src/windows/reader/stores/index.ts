import { defineStore } from 'pinia';
import { useBook } from '../lib';

export const useReaderStore = defineStore('reader', () => {
  const { pages, ...book } = useBook();

  function findNextIndex(current: number, step: number) {
    const index = pages.value.findIndex(({ id }) => id === current);
    return pages.value.at(index + step);
  }

  function lastIndex() {
    const index = pages.value.length - 1;
    return index >= 0 ? index : 0;
  }

  return {
    pages,
    current: book.current,
    findNextIndex,
    lastIndex,
    nextPage: book.next,
    previousPage: book.previous,
    firstPage: book.first,
    lastPage: book.last,
    reload: book.reload
  };
});
