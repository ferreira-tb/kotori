<script setup lang="ts">
import { Command } from '@/utils/commands';
import { Page } from './utils/page';

const readerId = window.__KOTORI__.readerId;
const { state: book } = useInvoke<Book | null>(Command.GetActiveBook, null, {
  args: { id: readerId }
});

// Pages and navigation.
const pages = ref<Page[]>([]);
const { state: current, prev, next, go } = useCycleList<Page>(pages);
onKeyDown('ArrowLeft', () => prev());
onKeyDown('ArrowRight', () => next());
onKeyDown('Home', () => go(0));
onKeyDown('End', () => go(pages.value.length - 1));

invokeOnKeyDown('Escape', Command.CloseWebviewWindow);
invokeOnKeyDown('Tab', Command.SwitchReaderFocus);
invokeOnKeyDown('F1', Command.FocusMainWindow);

whenever(book, (value) => {
  if (!value.pages) return;
  pages.value = Page.from(value.pages);
});

whenever(current, (value) => value.eagerFetch(pages.value));

onUnmounted(() => {
  for (const page of pages.value) {
    page.revoke();
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
