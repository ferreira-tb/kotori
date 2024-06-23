import { defineStore } from 'pinia';
import { Command } from '@/lib/commands';

export const useLibraryStore = defineStore('library', () => {
  const filter = ref('');
  const selected = ref<Nullish<LibraryBook>>();
  const books = useInvoke<LibraryBook[]>(Command.GetLibraryBooks, [], {
    lazy: true,
    transform(it) {
      return it.map((book) => {
        book.cover &&= convertFileSrc(book.cover);
        return book;
      });
    }
  });

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

      if (selected.value?.id === id) {
        selected.value = null;
      }
    }
  }

  let nextCoverVersion = 1;
  function updateBookCover(id: number, path: string) {
    const book = getBook(id);
    if (book) {
      try {
        // Adds a version search parameter to the url to force reload the image.
        // Without this, it would be cached, not updating when the user changes the cover.
        const url = new URL(convertFileSrc(path));
        url.searchParams.set('v', (++nextCoverVersion).toString(10));
        book.cover = url.toString();
      } catch {
        book.cover = convertFileSrc(path);
      } finally {
        triggerRef(books.state);
      }
    }
  }

  function updateBookRating(id: number, rating: number) {
    const book = getBook(id);
    if (book && isValidRating(book, rating)) {
      book.rating = rating;
      triggerRef(books.state);
    }
  }

  return {
    books: books.state,
    filter,
    selected,
    addBook,
    getBook,
    load: books.execute,
    removeBook,
    updateBookCover,
    updateBookRating
  };
});

function isValidRating(book: LibraryBook, value: number) {
  if (!Number.isInteger(value)) return false;
  return book.rating !== value && value >= 0 && value <= 5;
}
