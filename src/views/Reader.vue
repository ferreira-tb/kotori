<script setup lang="ts">
import { Command } from '@/utils';
import { useReaderStore } from '@/stores';
import { invoke } from '@tauri-apps/api/core';

const store = useReaderStore();
const { book } = storeToRefs(store);

const pages = computed(() => {
  if (!book.value) return [];
  return book.value.pages;
});

const showOnlyScaffoldContent = injectStrict(symbols.showOnlyScaffoldContent);

const { state: current, prev, next, go } = useCycleList(pages);
onKeyDown('ArrowLeft', () => prev());
onKeyDown('ArrowRight', () => next());
onKeyDown('Home', () => go(0));
onKeyDown('End', () => go(pages.value.length - 1));

const readerRef = shallowRef<HTMLElement | null>(null);
useEventListener(readerRef, 'dblclick', () => {
  showOnlyScaffoldContent.value = !showOnlyScaffoldContent.value;
});

onBeforeMount(() => {
  showOnlyScaffoldContent.value = true;
});

onBeforeUnmount(() => {
  showOnlyScaffoldContent.value = false;
});
</script>

<template>
  <div class="flex h-full w-full items-center justify-center">
    <div
      v-if="book && pages.length > 0"
      ref="readerRef"
      class="flex h-full w-full flex-col items-center justify-center overflow-hidden"
    >
      <img :src="current.path" class="h-full w-full object-contain" />
    </div>
    <div v-else>
      <m-button variant="outlined" @click="invoke(Command.OpenBook)">Open</m-button>
    </div>
  </div>
</template>
