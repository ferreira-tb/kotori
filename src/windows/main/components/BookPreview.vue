<script setup lang="ts">
import { getBookCover } from '@/utils/server';
import type { ImagePassThroughOptions } from 'primevue/image';

const props = defineProps<{
  readonly book: LibraryBook;
}>();

const preview = shallowRef<Blob | null>(null);
const previewUrl = useObjectUrl(preview);

const previewVisible = ref(false);
const loadingPreview = ref(false);

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
</script>

<template>
  <div class="flex h-full w-60 flex-col overflow-hidden">
    <p-image
      :pt
      preview
      :zoom-in-disabled="loadingPreview"
      :zoom-out-disabled="loadingPreview"
      class="self-center"
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
  </div>
</template>
