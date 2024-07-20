<script setup lang="ts">
import { LibraryMode } from '../router';
import { useLibraryStore } from '../stores';
import BookGrid from '../components/BookGrid.vue';

const store = useLibraryStore();
const { library, filter, selected } = storeToRefs(store);

const mode = useRouteQuery('mode');
const books = computed(() => {
  if (mode.value === LibraryMode.Favorites) {
    return library.value.favorites();
  }

  return library.value.all();
});
</script>

<template>
  <div class="size-full">
    <div v-if="library.size > 0" class="relative size-full overflow-hidden">
      <div class="absolute inset-0 overflow-y-auto overflow-x-hidden p-2">
        <BookGrid :books :filter @select="(it) => (selected = it)" />
      </div>
    </div>
  </div>
</template>
