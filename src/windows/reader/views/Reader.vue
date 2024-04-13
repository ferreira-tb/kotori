<script setup lang="ts">
import { showReaderPageContextMenu } from '@/utils/commands';
import { Page } from '../lib/page';
import { useReaderStore } from '../stores';

const store = useReaderStore();
const { pages, current } = storeToRefs(store);

onKeyDown('ArrowLeft', store.previousPage);
onKeyDown('ArrowRight', store.nextPage);
onKeyDown('Home', store.firstPage);
onKeyDown('End', store.lastPage);

onUnmounted(() => Page.revokeAll());
</script>

<template>
  <main class="fixed inset-0 overflow-hidden">
    <div class="flex size-full items-center justify-center">
      <div
        v-if="pages.length > 0 && current"
        class="flex size-full flex-col items-center justify-center"
      >
        <img
          v-if="current.status === 'done' && current.url"
          :src="current.url"
          class="size-full object-scale-down"
          @click="store.nextPage"
          @contextmenu="showReaderPageContextMenu(store.windowId, current?.id)"
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
