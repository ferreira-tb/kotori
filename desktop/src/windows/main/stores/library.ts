import { defineStore } from 'pinia';
import { Library, type LibraryBookImpl } from '../lib/library';

export const useLibraryStore = defineStore('library', () => {
  const filter = ref('');
  const library = Library.createRef();
  const selected = shallowRef<Nullish<LibraryBookImpl>>();

  return {
    library,
    filter,
    selected,
    load: () => library.value.load(),
  };
});
