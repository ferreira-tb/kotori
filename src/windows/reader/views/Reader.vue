<script setup lang="ts">
import { Page } from '../utils/page';
import { useReaderStore } from '../stores';

const store = useReaderStore();
const { pages } = storeToRefs(store);

// Pages and navigation.
const list = useCycleList<Page | null>(pages, { initialValue: null });
const current = list.state;

onKeyDown('ArrowLeft', () => list.prev());
onKeyDown('ArrowRight', () => list.next());
onKeyDown('Home', () => list.go(0));
onKeyDown('End', () => list.go(store.lastIndex()));

watchImmediate(current, (value) => value?.eagerFetch());

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
        />
      </div>
    </div>
  </main>
</template>
