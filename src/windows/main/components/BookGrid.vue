<script setup lang="ts">
import { contains } from '@/utils/string';
import { openBook, showLibraryBookContextMenu } from '@/utils/commands';
import { useLibraryStore } from '../stores';

defineEmits<(e: 'select', book: LibraryBook) => void>();

const store = useLibraryStore();
const { books, filter } = storeToRefs(store);
</script>

<template>
  <div id="kt-book-grid">
    <template v-for="book of books" :key="book.id">
      <div
        v-if="book.cover && contains(filter, book.title)"
        class="cursor-pointer overflow-hidden rounded-sm"
        @click="$emit('select', book)"
        @dblclick="openBook(book.id)"
        @contextmenu="showLibraryBookContextMenu(book.id)"
      >
        <img :src="book.cover" class="size-full object-cover" />
      </div>
    </template>
  </div>
</template>

<style scoped>
#kt-book-grid {
  display: grid;
  grid-template-columns: repeat(10, minmax(0, 1fr));
  gap: 1rem;
}
</style>
