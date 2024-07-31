<script setup lang="ts">
import { Reader } from '../lib/reader';
import { useReaderStore } from '../stores';
import { BookPageStatus } from '../lib/page';
import { showReaderPageContextMenu } from '@/lib/commands';

const store = useReaderStore();
const { reader } = storeToRefs(store);
void reader.value.load();

onKeyDown('ArrowUp', () => reader.value.previous());
onKeyDown('ArrowLeft', () => reader.value.previous());
onKeyDown('ArrowDown', () => reader.value.next());
onKeyDown('ArrowRight', () => reader.value.next());
onKeyDown('Home', () => reader.value.first());
onKeyDown('End', () => reader.value.last());
onKeyDown('Delete', () => reader.value.current?.delete());

// This will need to be updated to support scrolling.
useEventListener(window, 'wheel', (event: WheelEvent) => {
  if (event.deltaY < 0) {
    reader.value.previous();
  }
  else {
    reader.value.next();
  }
});

function showContextMenu() {
  const name = reader.value.current?.name;
  if (name) {
    showReaderPageContextMenu(Reader.windowId, name);
  }
}
</script>

<template>
  <div class="flex size-full items-center justify-center">
    <div
      v-if="reader.size > 0 && reader.current"
      class="flex size-full flex-col items-center justify-center"
    >
      <img
        v-if="reader.current.status === BookPageStatus.Done && reader.current.url"
        :src="reader.current.url"
        class="size-full object-scale-down"
        @contextmenu="showContextMenu"
      >
    </div>
  </div>
</template>
