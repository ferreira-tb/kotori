<script setup lang="ts">
import { Button } from '@/components/ui/button';
// import { getBookCover } from '@/lib/server';
import { openBook, updateBookRating } from '@/lib/commands';

const props = defineProps<{
  book: LibraryBook;
}>();
/*
const preview = shallowRef<Blob | null>(null);
const previewUrl = useObjectUrl(preview);
const loadingPreview = ref(false);*/

const rating = ref(0);
const ratingWatcher = watchPausable(rating, updateRating);
watchImmediate(() => props.book, setRating);
/*
async function fetchPreview() {
  if (loadingPreview.value) return;
  try {
    loadingPreview.value = true;
    preview.value = await getBookCover(props.book.id);
  } catch (err) {
    handleError(err);
  } finally {
    loadingPreview.value = false;
  }
}*/

async function setRating() {
  ratingWatcher.pause();
  await nextTick();
  rating.value = props.book.rating;

  await nextTick();
  ratingWatcher.resume();
}

function updateRating(value: number) {
  updateBookRating(props.book.id, value);
}
</script>

<template>
  <div class="flex h-full w-60 flex-col items-center gap-4 overflow-hidden">
    <img v-if="book.cover" :src="book.cover" :alt="book.title" class="w-56 object-scale-down" />
    <!-- p-rating v-model="rating" :cancel="false" /-->
    <div class="w-full text-center">{{ book.title }}</div>
    <div class="flex gap-4">
      <Button @click="openBook(book.id)">Open</Button>
    </div>
  </div>
</template>
