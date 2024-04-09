<script setup lang="ts">
import { openBook, showLibraryBookContextMenu } from '@/utils/commands';

defineProps<{
  readonly books: LibraryBook[];
}>();

defineEmits<(e: 'select', book: LibraryBook) => void>();
</script>

<template>
  <div class="book-grid">
    <template v-for="book of books" :key="book.id">
      <div
        v-if="book.cover"
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
.book-grid {
  display: grid;
  grid-template-columns: repeat(10, minmax(0, 1fr));
  gap: 1rem;
}
</style>
