import { defineStore } from 'pinia';
import { Command } from '@/utils/commands';
import { convertFileSrc } from '@tauri-apps/api/core';

export const useLibraryStore = defineStore('library', () => {
  const books = useInvoke<LibraryBook[]>(Command.GetLibraryBooks, [], {
    transform,
    shallow: false
  });

  function getBook(id: number) {
    return books.state.value.find((book) => book.id === id);
  }

  function updateBookCover(id: number, cover: string) {
    const book = getBook(id);
    if (book) book.cover = convertFileSrc(cover);
  }

  return {
    books: books.state,
    updateBookCover
  };
});

function transform(books: LibraryBook[]) {
  return books.map((book) => {
    book.cover &&= convertFileSrc(book.cover);
    return book;
  });
}
