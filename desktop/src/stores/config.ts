import { defineStore } from 'pinia';

export const useConfigStore = defineStore('config', () => {
  const colorMode = useColorMode({ storageKey: null });

  return {
    colorMode,
  };
});
