<script setup lang="ts">
import { useReaderStore } from '../stores';
import { READER_WINDOW_ID } from '../lib/global';
import { showReaderPageContextMenu, toggleReaderMenu } from '@/lib/commands';

const store = useReaderStore();
const { pages, current } = storeToRefs(store);

// Using `alt` doesn't seem to be a good idea.
// What could be a better way to toggle the menu?
onAltKeyDown('Alt', toggleReaderMenu);

onKeyDown('ArrowUp', store.previousPage);
onKeyDown('ArrowLeft', store.previousPage);
onKeyDown('ArrowDown', store.nextPage);
onKeyDown('ArrowRight', store.nextPage);
onKeyDown('Home', store.firstPage);
onKeyDown('End', store.lastPage);
onKeyDown('Delete', () => current.value?.delete());

// This will need to be updated to support scrolling.
useEventListener(window, 'wheel', (event: WheelEvent) => {
  if (event.deltaY < 0) {
    store.previousPage();
  } else {
    store.nextPage();
  }
});
</script>

<template>
  <main class="fixed inset-0 select-none overflow-hidden">
    <div class="flex size-full items-center justify-center">
      <div
        v-if="pages.length > 0 && current"
        class="flex size-full flex-col items-center justify-center"
      >
        <img
          v-if="current.status === 'done' && current.url"
          :src="current.url"
          class="size-full object-scale-down"
          @contextmenu="showReaderPageContextMenu(READER_WINDOW_ID, current?.id)"
        />
        <p-progress-spinner
          v-else-if="current.status === 'pending'"
          class="size-16"
          stroke-width="4"
        />
      </div>
    </div>
  </main>
</template>
