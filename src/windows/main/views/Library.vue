<script setup lang="ts">
import { requestRemoveBook } from '@/utils/commands';
import { symbols } from '../utils/symbols';
import { useLibraryStore } from '../stores';
import BookGrid from '../components/BookGrid.vue';
import BookPreview from '../components/BookPreview.vue';
import DialogRemoveBook from '../components/dialog/RemoveBook.vue';

const store = useLibraryStore();
const { books, filter } = storeToRefs(store);

const filtered = computed(() => {
  const lowercase = filter.value.toLowerCase();
  return store.books.filter((book) => {
    if (!book.cover) return false;
    return book.title.toLowerCase().includes(lowercase);
  });
});

const preview = ref<Nullish<LibraryBook>>(null);
watchEffect(() => {
  if (books.value.every((book) => !book.cover)) {
    preview.value = null;
  } else if (preview.value && books.value.every(({ id }) => id !== preview.value?.id)) {
    preview.value = books.value.find((book) => book.cover);
  } else {
    preview.value ??= books.value.find((book) => book.cover);
  }
});

const contentHeight = injectStrict(symbols.contentHeight);

onKeyDown('Delete', () => requestRemoveBook(preview.value?.id));
</script>

<template>
  <div class="size-full select-none">
    <!-- We use `books` instead of `filtered` to show the preview even when the filter hides all books -->
    <div v-if="books.length > 0" class="relative size-full overflow-hidden">
      <!-- Using `key` ensures the preview is updated when the cover changes -->
      <book-preview v-if="preview && preview.cover" :key="preview.cover" :book="preview" />
      <div
        v-if="filtered.length > 0"
        class="absolute bottom-0 left-60 top-0 overflow-y-auto overflow-x-hidden px-2 pb-2"
      >
        <book-grid :books="filtered" @select="(book) => (preview = book)" />
      </div>
    </div>

    <dialog-remove-book />
  </div>
</template>

<style scoped>
div:has(> .book-grid) {
  height: v-bind('contentHeight');
}
</style>
