<script setup lang="ts">
import { requestRemoveBook } from '@/utils/commands';
import { RouteName } from '../router';
import { symbols } from '../utils/symbols';
import { useLibraryStore } from '../stores';
import BookGrid from '../components/BookGrid.vue';
import BookPreview from '../components/BookPreview.vue';
import DialogRemoveBook from '../components/dialog/RemoveBook.vue';

const store = useLibraryStore();
const { books, filter } = storeToRefs(store);

const route = useRoute();
const preview = ref<Nullish<LibraryBook>>(null);
const contentHeight = injectStrict(symbols.contentHeight);

watchEffect(() => {
  if (books.value.every((book) => !book.cover)) {
    preview.value = null;
  } else if (preview.value && books.value.every(({ id }) => id !== preview.value?.id)) {
    preview.value = books.value.find((book) => book.cover);
  } else {
    preview.value ??= books.value.find((book) => book.cover);
  }
});

onKeyDown('Delete', () => requestRemoveBook(preview.value?.id));
</script>

<template>
  <teleport v-if="route.name === RouteName.Library" to="#kt-menubar-end">
    <div>
      <p-icon-field icon-position="left">
        <p-input-icon class="pi pi-search" />
        <p-input-text v-model="filter" size="small" placeholder="Search" spellcheck="false" />
      </p-icon-field>
    </div>
  </teleport>

  <div class="size-full select-none">
    <div v-if="books.length > 0" class="relative size-full overflow-hidden">
      <!-- Using `key` ensures the preview is updated when the cover changes -->
      <book-preview v-if="preview && preview.cover" :key="preview.cover" :book="preview" />
      <div class="absolute bottom-0 left-60 top-0 overflow-y-auto overflow-x-hidden px-2 pb-2">
        <book-grid @select="(book) => (preview = book)" />
      </div>
    </div>

    <dialog-remove-book />
  </div>
</template>

<style scoped>
div:has(> #kt-book-grid) {
  height: v-bind('contentHeight');
}
</style>
