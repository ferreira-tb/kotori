<script setup lang="ts">
import { symbols } from '../utils/symbols';
import { useLibraryStore } from '../stores';
import BookGrid from '../components/BookGrid.vue';
import BookPreview from '../components/BookPreview.vue';

const store = useLibraryStore();

const contentHeight = injectStrict(symbols.contentHeight);

const { filter } = storeToRefs(store);
const books = computed(() => {
  const lowercase = filter.value.toLowerCase();
  return store.books.filter((book) => {
    if (!book.cover) return false;
    return book.title.toLowerCase().includes(lowercase);
  });
});

const selected = ref<LibraryBook | null>(null);
const preview = computed(() => selected.value ?? store.books[0]);
</script>

<template>
  <div class="size-full">
    <!-- We use `store.books` instead of `books` to show the preview even when the filter hides all books -->
    <div v-if="store.books.length > 0" class="relative size-full overflow-hidden">
      <book-preview v-if="preview && preview.cover" :book="preview" />
      <div
        v-if="books.length > 0"
        class="absolute bottom-0 left-60 top-0 overflow-y-auto overflow-x-hidden px-2 pb-2"
      >
        <book-grid :books @select="(book) => (selected = book)" />
      </div>
    </div>
  </div>
</template>

<style scoped>
div:has(> .book-grid) {
  height: v-bind('contentHeight');
}
</style>
