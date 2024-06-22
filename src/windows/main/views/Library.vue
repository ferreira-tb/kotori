<script setup lang="ts">
import { RouteName } from '../router';
import { useLibraryStore } from '../stores';
import BookGrid from '../components/BookGrid.vue';
import { symbols } from '@/windows/main/lib/symbols';
import { removeBook, usePreview } from '../lib/book';
import BookPreview from '../components/BookPreview.vue';

const store = useLibraryStore();
const { books, filter } = storeToRefs(store);

const route = useRoute();
const preview = usePreview();
const contentHeight = injectStrict(symbols.contentHeight);

onKeyDown('Delete', () => removeBook(preview.value?.id));
</script>

<template>
  <!--teleport v-if="route.name === RouteName.Library" to="#kt-menubar-end">
    <div>
      <p-icon-field icon-position="left">
        <p-input-icon class="pi pi-search" />
        <p-input-text v-model="filter" size="small" placeholder="Search" spellcheck="false" />
      </p-icon-field>
    </div>
  </teleport -->

  <div class="size-full">
    <div v-if="books.length > 0" class="relative size-full overflow-hidden">
      <!-- Using `key` ensures the preview is updated when the cover changes -->
      <BookPreview v-if="preview && preview.cover" :key="preview.cover" :book="preview" />
      <div class="absolute inset-y-0 left-60 overflow-y-auto overflow-x-hidden p-2 pt-0">
        <BookGrid @select="(book) => (preview = book)" />
      </div>
    </div>
  </div>
</template>
