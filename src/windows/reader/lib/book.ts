import { Command } from '@/utils/commands';
import { Page } from './page';

export function useBook() {
  const book = useInvoke<ReaderBook | null>(Command.GetCurrentReaderBook, null);

  const pages = ref<Page[]>([]);
  watch(book.state, (value) => {
    pages.value = value?.pages.map((id) => new Page(id)) ?? [];
  });

  const currentIndex = ref(0);
  const current = computed(() => pages.value.at(currentIndex.value));
  watchImmediate(current, (value) => value?.eagerFetch());

  function go(page: number) {
    const len = pages.value.length;
    currentIndex.value = ((page % len) + len) % len;
  }

  function next() {
    go(currentIndex.value + 1);
  }

  function previous() {
    go(currentIndex.value - 1);
  }

  function first() {
    go(0);
  }

  function last() {
    go(pages.value.length - 1);
  }

  return {
    pages,
    current,
    next,
    previous,
    first,
    last,
    reload: book.execute
  };
}
