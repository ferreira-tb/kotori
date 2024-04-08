import { triggerRef } from 'vue';
import { defineStore } from 'pinia';
import { Command } from '@/utils/commands';
import { convertFileSrc } from '@tauri-apps/api/core';

export const useLibraryStore = defineStore('library', () => {
  const books = useInvoke<LibraryBook[]>(Command.GetLibraryBooks, [], { transform });

  function addBook(book: LibraryBook) {
    books.state.value.push(book);
    triggerRef(books.state);
  }

  function getBook(id: number) {
    return books.state.value.find((book) => book.id === id);
  }

  function updateBookCover(id: number, path: string) {
    const book = getBook(id);
    if (book) {
      book.cover = convertFileSrc(path);
      triggerRef(books.state);
    }
  }

  function updateBookRating(id: number, rating: number) {
    const book = getBook(id);
    if (book && isValidRating(book.rating, rating)) {
      book.rating = rating;
      triggerRef(books.state);
    }
  }

  return {
    books: books.state,
    addBook,
    updateBookCover,
    updateBookRating
  };
});

function transform(books: LibraryBook[]) {
  return books.map((book) => {
    book.cover &&= convertFileSrc(book.cover);
    return book;
  });
}

function isValidRating(current: number, next: number) {
  if (!Number.isInteger(next)) return false;
  return current !== next && next >= 0 && next <= 5;
}
