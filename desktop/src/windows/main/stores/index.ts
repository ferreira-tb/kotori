import { useLibraryStore } from './library';
import { useConfigStore } from '@/stores/config';
import { useCollectionStore } from './collection';

export * from './library';
export * from './collection';

export function loadStores() {
  return Promise.all([
    useConfigStore().$tauri.start(),
    useLibraryStore().load(),
    useCollectionStore().load()
  ]);
}
