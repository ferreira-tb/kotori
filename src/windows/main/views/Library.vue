<script setup lang="ts">
import { openBook } from '@/utils/commands';
import { useLibraryStore } from '../stores';

const store = useLibraryStore();
const { books } = storeToRefs(store);

watchEffect(() => {
  console.log('Books:', books.value);
});
</script>

<template>
  <main>
    <div class="grid grid-cols-8 gap-4">
      <div v-for="book of books" :key="book.id" class="cursor-pointer" @click="openBook(book.id)">
        <img v-if="book.cover" :src="book.cover" class="size-full object-cover" />
      </div>
    </div>
  </main>
</template>
