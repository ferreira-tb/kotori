<script setup lang="ts">
import { contains } from '@/lib/string';
import { useLibraryStore } from '../stores';
import { openBook, showLibraryBookContextMenu } from '@/lib/commands';

defineEmits<(e: 'select', book: LibraryBook) => void>();

const store = useLibraryStore();
const { books, filter } = storeToRefs(store);

async function open(id: number) {
  try {
    await openBook(id);
  } catch (err) {
    handleError(err, { dialog: true });
  }
}
</script>

<template>
  <div id="kt-book-grid">
    <template v-for="book of books" :key="book.id">
      <div
        v-if="book.cover && contains(filter, book.title)"
        class="cursor-pointer overflow-hidden rounded-sm"
        @click="$emit('select', book)"
        @dblclick="open(book.id)"
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
