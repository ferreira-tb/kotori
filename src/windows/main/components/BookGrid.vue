<script setup lang="ts">
import { contains } from '@/lib/string';
import type { LibraryBookImpl } from '../lib/library';
import { showLibraryBookContextMenu } from '@/lib/commands';

defineProps<{
  books: Iterable<LibraryBookImpl>;
  filter: string;
}>();

defineEmits<(e: 'select', book: LibraryBookImpl) => void>();
</script>

<template>
  <div class="grid gap-4 sm:grid-cols-4 md:grid-cols-6 lg:grid-cols-8 xl:grid-cols-10">
    <template v-for="book of books" :key="book.id">
      <div
        v-if="book.cover && contains(filter, book.title)"
        class="cursor-pointer overflow-hidden rounded-md border border-solid shadow-md"
        @click="$emit('select', book)"
        @dblclick="book.open()"
        @contextmenu="showLibraryBookContextMenu(book.id)"
      >
        <img
          :src="book.cover"
          :alt="book.title"
          class="h-auto w-auto object-cover transition-all hover:scale-110"
        />
      </div>
    </template>
  </div>
</template>
