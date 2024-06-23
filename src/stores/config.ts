import { defineStore } from 'pinia';
import { Store } from '@tauri-apps/plugin-store';
import type { BasicColorSchema } from '@vueuse/core';

enum Key {
  ColorMode = 'color-mode'
}

export const useConfigStore = defineStore('config', () => {
  const store = new Store('config.json');

  const colorMode = useColorMode();

  const handlers: ReturnType<typeof watchPausable>[] = [];
  handlers.push(watchPausable(colorMode, onChange(Key.ColorMode)));

  async function load() {
    handlers.forEach((it) => it.pause());
    await nextTick();

    colorMode.value = (await store.get(Key.ColorMode)) ?? 'auto';

    await nextTick();
    handlers.forEach((it) => it.resume());
  }

  function save() {
    store.save().catch(handleError);
  }

  function onChange(key: Key) {
    return function <T>(value: T) {
      store.set(key, value).catch(handleError);
    };
  }

  return {
    colorMode: colorMode as Ref<BasicColorSchema>,
    load,
    save
  };
});
