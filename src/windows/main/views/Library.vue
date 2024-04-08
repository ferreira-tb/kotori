<script setup lang="ts">
import { toPixel } from '@tb-dev/utils';
import { openBook } from '@/utils/commands';
import { useLibraryStore } from '../stores';

const store = useLibraryStore();

const menubar = shallowRef<HTMLElement | null>(null);
const { height: menubarHeight } = useElementSize(menubar);
const { height: windowHeight } = useWindowSize();

const filter = ref('');
const books = computed(() => {
  const lowercase = filter.value.toLowerCase();
  return store.books.filter((book) => book.title.toLowerCase().includes(lowercase));
});

watchEffect(() => {
  for (const book of books.value) {
    console.log(book.cover);
  }
});
</script>

<template>
  <main class="fixed inset-0">
    <div ref="menubar" class="absolute inset-x-0 top-0">
      <p-menubar class="border-none">
        <template #end>
          <p-input-text v-model="filter" size="small" placeholder="Search" spellcheck="false" />
        </template>
      </p-menubar>
    </div>
    <div>
      <div v-if="books.length > 0" class="book-grid">
        <template v-for="book of books" :key="book.id">
          <div
            v-if="book.cover"
            class="cursor-pointer overflow-hidden rounded-sm"
            @click="openBook(book.id)"
          >
            <img :src="book.cover" class="size-full object-cover" />
          </div>
        </template>
      </div>
    </div>
  </main>
</template>

<style scoped>
div:has(> .book-grid) {
  position: relative;
  top: v-bind('toPixel(menubarHeight)');
  padding: 0 0.5rem 0.5rem;
  width: 100%;
  height: v-bind('toPixel(windowHeight - menubarHeight)');
  overflow-x: hidden;
  overflow-y: auto;
}

.book-grid {
  display: grid;
  grid-template-columns: repeat(10, minmax(100px, 1fr));
  gap: 1rem;
}
</style>
