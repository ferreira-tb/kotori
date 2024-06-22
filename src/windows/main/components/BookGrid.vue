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
  <div class="grid gap-4 sm:grid-cols-4 md:grid-cols-6 lg:grid-cols-8 xl:grid-cols-10">
    <template v-for="book of books" :key="book.id">
      <div
        v-if="book.cover && contains(filter, book.title)"
        class="cursor-pointer overflow-hidden rounded-sm"
        @click="$emit('select', book)"
        @dblclick="open(book.id)"
        @contextmenu="showLibraryBookContextMenu(book.id)"
      >
        <img :src="book.cover" :alt="book.title" class="size-full object-cover" />
      </div>
    </template>
  </div>
</template>
