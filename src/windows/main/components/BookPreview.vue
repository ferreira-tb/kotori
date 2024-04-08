<script setup lang="ts">
import { getBookCover } from '@/utils/server';
import { updateBookRating } from '@/utils/commands';
import type { ImagePassThroughOptions } from 'primevue/image';

const props = defineProps<{
  readonly book: LibraryBook;
}>();

const preview = shallowRef<Blob | null>(null);
const previewUrl = useObjectUrl(preview);

const previewVisible = ref(false);
const loadingPreview = ref(false);

const rating = ref(0);
const ratingWatcher = watchPausable(rating, updateRating);
watchImmediate(() => props.book, setRating);

const pt = computed<ImagePassThroughOptions>(() => {
  const buttonClass = previewVisible.value && !loadingPreview.value ? '' : 'hidden';
  const toolbarClass = previewVisible.value ? '' : 'hidden';

  return {
    rotateLeftButton: { class: buttonClass },
    rotateRightButton: { class: buttonClass },
    zoomInButton: { class: buttonClass },
    zoomOutButton: { class: buttonClass },
    toolbar: { class: toolbarClass }
  };
});

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
}

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
    <p-image
      :pt
      preview
      :zoom-in-disabled="loadingPreview"
      :zoom-out-disabled="loadingPreview"
      @click="fetchPreview"
      @show="previewVisible = true"
      @hide="previewVisible = false"
    >
      <template #image>
        <img :src="book.cover" :alt="book.title" class="w-56 object-scale-down" />
      </template>
      <template #preview="slotProps">
        <p-progress-spinner v-if="loadingPreview" class="size-16" stroke-width="4" />
        <div
          v-else
          :key="book.id"
          class="flex h-screen w-screen items-start justify-center overflow-auto p-4"
        >
          <img
            :src="previewUrl"
            :alt="book.title"
            :style="slotProps.style"
            class="object-scale-down"
          />
        </div>
      </template>
    </p-image>

    <p-rating v-model="rating" :cancel="false" />
    <div class="w-full text-center">{{ book.title }}</div>
  </div>
</template>
