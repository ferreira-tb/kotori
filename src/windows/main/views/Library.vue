<script setup lang="ts">
import { LibraryMode } from '../router';
import { useLibraryStore } from '../stores';
import BookGrid from '../components/BookGrid.vue';

const library = useLibraryStore();
const { filter, selected } = storeToRefs(library);

const mode = useRouteQuery('mode');
const books = computed(() => {
  if (mode.value === LibraryMode.Favorites) {
    return library.books.filter((it) => it.rating >= 4);
  }

  return library.books;
});
</script>

<template>
  <div class="size-full">
    <div v-if="books.length > 0" class="relative size-full overflow-hidden">
      <div class="absolute inset-0 overflow-y-auto overflow-x-hidden p-2">
        <BookGrid :books :filter @select="(it) => (selected = it)" />
      </div>
    </div>
  </div>
</template>
