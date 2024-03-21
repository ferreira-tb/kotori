import { defineStore } from 'pinia';

export const useReaderStore = defineStore('reader', () => {
  const book = shallowRef<Book | null>(null);

  return {
    book
  };
});
