import { Book } from '../lib/book';
import { defineStore } from 'pinia';

export const useReaderStore = defineStore('reader', () => {
  const book = new Book();

  return {
    pages: book.pages,
    current: book.current,
    ready: book.ready,
    findNextIndex: book.peek.bind(book),
    lastIndex: book.lastIndex.bind(book),
    nextPage: book.next.bind(book),
    previousPage: book.previous.bind(book),
    firstPage: book.first.bind(book),
    lastPage: book.last.bind(book),
    load: book.load.bind(book)
  };
});
