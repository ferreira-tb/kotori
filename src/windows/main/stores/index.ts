import { useLibraryStore } from './library';
import { useConfigStore } from '@/stores/config';
import { useCollectionStore } from './collection';

export * from './library';
export * from './collection';

export function loadStores() {
  const config = useConfigStore();
  const library = useLibraryStore();
  const collections = useCollectionStore();

  return Promise.all([config.load(), library.load(), collections.load()]);
}
