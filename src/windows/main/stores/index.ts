import { triggerRef } from 'vue';
import { defineStore } from 'pinia';
import { Command } from '@/utils/commands';
import { convertFileSrc } from '@tauri-apps/api/core';

export const useLibraryStore = defineStore('library', () => {
  const books = useInvoke<LibraryBook[]>(Command.GetLibraryBooks, [], { transform });
  const filter = ref('');

  function addBook(book: LibraryBook) {
    book.cover &&= convertFileSrc(book.cover);
    books.state.value.push(book);
    triggerRef(books.state);
  }

  function getBook(id: number) {
    return books.state.value.find((book) => book.id === id);
  }

  function removeBook(id: number) {
    const index = books.state.value.findIndex((book) => book.id === id);
    if (index !== -1) {
      books.state.value.splice(index, 1);
      triggerRef(books.state);
    }
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
    filter,
    addBook,
    removeBook,
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
