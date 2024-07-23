import { defineStore } from 'pinia';
import { Reader } from '../lib/reader';

export const useReaderStore = defineStore('reader', () => {
  const reader = Reader.createRef();

  return {
    reader,
  };
});
