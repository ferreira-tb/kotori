import { defineStore } from 'pinia';
import { Command } from '@/utils/commands';

export const useLibraryStore = defineStore('library', () => {
  const books = useInvoke<ReaderBook[]>(Command.GetLibraryBooks, []);

  return {
    books: books.state
  };
});
