<script setup lang="ts">
import { toPixel } from '@tb-dev/utils';
import { useLibraryStore } from '../stores';
import BookGrid from '../components/BookGrid.vue';
import BookPreview from '../components/BookPreview.vue';

const store = useLibraryStore();

const menubar = shallowRef<HTMLElement | null>(null);
const { height: menubarHeight } = useElementSize(menubar);
const { height: windowHeight } = useWindowSize();

const contentHeight = computed(() => {
  return toPixel(windowHeight.value - menubarHeight.value);
});

const filter = ref('');
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
  <main class="fixed inset-0">
    <div ref="menubar" class="absolute inset-x-0 top-0">
      <p-menubar class="rounded-none border-none">
        <template #end>
          <p-input-text v-model="filter" size="small" placeholder="Search" spellcheck="false" />
        </template>
      </p-menubar>
    </div>
    <div class="libray-content">
      <div v-if="store.books.length > 0" class="relative overflow-hidden">
        <book-preview v-if="preview" :book="preview" />
        <div
          v-if="books.length > 0"
          class="absolute bottom-0 left-60 top-0 overflow-y-auto overflow-x-hidden px-2 pb-2"
        >
          <book-grid :books @select="(book) => (selected = book)" />
        </div>
      </div>
    </div>
  </main>
</template>

<style scoped>
.libray-content {
  position: relative;
  top: v-bind('toPixel(menubarHeight)');
  padding: 0 0.5rem 0.5rem;
  width: 100%;
  height: v-bind('contentHeight');
  overflow: hidden;
}

.libray-content > div:has(.book-grid) {
  height: v-bind('contentHeight');
}
</style>
