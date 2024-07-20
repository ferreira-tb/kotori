import { defineStore } from 'pinia';
import { Command } from '@/lib/commands';

export const useCollectionStore = defineStore('collection', () => {
  const collections = useInvoke<BookCollection[]>(Command.GetCollections, [], {
    lazy: true
  });

  return {
    collections: collections.state,
    load: collections.execute
  };
});
