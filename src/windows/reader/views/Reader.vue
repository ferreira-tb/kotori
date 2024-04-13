<script setup lang="ts">
import { requestDeletePage, showReaderPageContextMenu } from '@/utils/commands';
import { Page } from '../lib/page';
import { useReaderStore } from '../stores';
import DialogDeletePage from '../components/dialog/DeletePage.vue';

const store = useReaderStore();
const { windowId, pages, current } = storeToRefs(store);

const dialogDeletePage = ref(false);
const enableKeydown = computed(() => !dialogDeletePage.value);

onKeyDown('ArrowUp', store.previousPage, { enabled: enableKeydown });
onKeyDown('ArrowLeft', store.previousPage, { enabled: enableKeydown });
onKeyDown('ArrowDown', store.nextPage, { enabled: enableKeydown });
onKeyDown('ArrowRight', store.nextPage, { enabled: enableKeydown });
onKeyDown('Home', store.firstPage, { enabled: enableKeydown });
onKeyDown('End', store.lastPage, { enabled: enableKeydown });
onKeyDown('Delete', deletePage, { enabled: enableKeydown });

function deletePage() {
  requestDeletePage(windowId.value, current.value?.id);
}

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

    <dialog-delete-page v-model="dialogDeletePage" />
  </main>
</template>
