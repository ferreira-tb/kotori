<script setup lang="ts">
import { showReaderPageContextMenu } from '@/utils/commands';
import { Event } from '../events';
import { Page } from '../lib/page';
import { useReaderStore } from '../stores';
import DialogDeletePage from '../components/DialogDeletePage.vue';

const store = useReaderStore();
const { pages, current } = storeToRefs(store);

const dialogDeletePage = ref(false);
const enableKeydown = computed(() => !dialogDeletePage.value);

onKeyDown('ArrowUp', store.previousPage, { enabled: enableKeydown });
onKeyDown('ArrowLeft', store.previousPage, { enabled: enableKeydown });
onKeyDown('ArrowDown', store.nextPage, { enabled: enableKeydown });
onKeyDown('ArrowRight', store.nextPage, { enabled: enableKeydown });
onKeyDown('Home', store.firstPage, { enabled: enableKeydown });
onKeyDown('End', store.lastPage, { enabled: enableKeydown });

useListen(Event.DeletePageRequested, () => {
  dialogDeletePage.value = true;
});

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
          @contextmenu="showReaderPageContextMenu(store.windowId, current?.id)"
        />
        <p-progress-spinner
          v-else-if="current.status === 'pending'"
          class="size-16"
          stroke-width="4"
        />
      </div>
    </div>

    <dialog-delete-page v-if="current" v-model="dialogDeletePage" :page="current.id" />
  </main>
</template>
