<script setup lang="ts">
import { Command } from '@/utils/commands';
import { Page } from './utils/page';

const readerId = window.__KOTORI__.readerId;
const { state: book } = useInvoke<Book | null>(Command.GetActiveBook, null, {
  args: { id: readerId }
});

const pages = ref<Page[]>([]);

const { state: current, prev, next, go } = useCycleList(pages);
onKeyDown('ArrowLeft', () => prev());
onKeyDown('ArrowRight', () => next());
onKeyDown('Home', () => go(0));
onKeyDown('End', () => go(pages.value.length - 1));

watchEffect(() => {
  if (!book.value?.pages) return;
  pages.value = Page.from(book.value.pages);
});

watchImmediate(current, (value) => {
  if (pages.value.length === 0) return;

  const promises: Promise<void>[] = [];
  if (value.status === 'not started') {
    promises.push(fetchPage(value));
  }

  for (let i = 1; i <= 5; i++) {
    const nextIndex = findIndex(value.id) + i;
    if (pages.value[nextIndex] && pages.value[nextIndex].status === 'not started') {
      promises.push(fetchPage(pages.value[nextIndex]));
    }
  }

  Promise.all(promises).catch(handleError);
});

async function fetchPage(page: Page) {
  try {
    console.log('page', page);
    if (!pages.value.includes(page) || page.status !== 'not started') return;
    page.status = 'pending';

    const url = 'http://localhost:3000/reader';
    const response = await fetch(`${url}/${readerId}/${page.id}`);
    const blob = await response.blob();

    page.url = URL.createObjectURL(blob);
    page.status = 'done';
  } catch (err) {
    page.status = 'error';
    handleError(err);
  }
}

function findIndex(id: number) {
  return pages.value.findIndex((page) => page.id === id);
}

onUnmounted(() => {
  for (const page of pages.value) {
    if (page.url) URL.revokeObjectURL(page.url);
  }
});
</script>

<template>
  <m-scaffold>
    <div class="flex size-full items-center justify-center">
      <div
        v-if="pages.length > 0"
        class="flex size-full flex-col items-center justify-center overflow-hidden"
      >
        <img
          v-if="current.status === 'done' && current.url"
          :src="current.url"
          class="size-full object-contain"
        />
      </div>
    </div>
  </m-scaffold>
</template>
