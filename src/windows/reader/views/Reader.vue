<script setup lang="ts">
import { requestDeletePage, showReaderPageContextMenu, showWindow } from '@/utils/commands';
import { READER_WINDOW_ID } from '../lib';
import { useReaderStore } from '../stores';
import DialogDeletePage from '../components/dialog/DeletePage.vue';

const store = useReaderStore();
const { pages, current } = storeToRefs(store);

const dialogDeletePage = ref(false);
const enableListeners = computed(() => !dialogDeletePage.value);

onKeyDown('ArrowUp', store.previousPage, { enabled: enableListeners });
onKeyDown('ArrowLeft', store.previousPage, { enabled: enableListeners });
onKeyDown('ArrowDown', store.nextPage, { enabled: enableListeners });
onKeyDown('ArrowRight', store.nextPage, { enabled: enableListeners });
onKeyDown('Home', store.firstPage, { enabled: enableListeners });
onKeyDown('End', store.lastPage, { enabled: enableListeners });
onKeyDown('Delete', deletePage, { enabled: enableListeners });

// This will need to be updated to support scrolling.
useEventListener(globalThis, 'wheel', (event: WheelEvent) => {
  if (!enableListeners.value) return;
  if (event.deltaY < 0) {
    store.previousPage();
  } else {
    store.nextPage();
  }
});

function deletePage() {
  requestDeletePage(READER_WINDOW_ID, current.value?.id);
}

onMounted(() => {
  until(current).toBeTruthy().then(showWindow).catch(handleError);
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

    <dialog-delete-page v-model="dialogDeletePage" />
  </main>
</template>
