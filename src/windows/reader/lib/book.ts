import { Page } from './page';
import { getCurrentReaderBook } from '@/lib/commands';

export class Book {
  readonly #book = ref<ReaderBook | null>(null);
  readonly #ready = ref(false);
  readonly #currentIndex = ref(0);

  public readonly ready = readonly(this.#ready);
  public readonly pages = ref<Page[]>([]);
  public readonly current = computed<Nullish<Page>>(() => {
    return this.pages.value.at(this.#currentIndex.value);
  });

  constructor() {
    watch(this.#book, (value) => {
      this.pages.value = value?.pages.map((id) => new Page(id)) ?? [];
    });

    watchImmediate(this.current, (value) => value?.eagerFetch());

    this.load().catch(handleError);
  }

  public go(page: number) {
    const len = this.pages.value.length;
    this.#currentIndex.value = ((page % len) + len) % len;
  }

  public next() {
    this.go(this.#currentIndex.value + 1);
  }

  public previous() {
    this.go(this.#currentIndex.value - 1);
  }

  public first() {
    this.go(0);
  }

  public last() {
    this.go(this.lastIndex());
  }

  public lastIndex() {
    const index = this.pages.value.length - 1;
    return index >= 0 ? index : 0;
  }

  public peek(page: number, offset = 0): Page | null {
    const index = this.pages.value.findIndex(({ id }) => id === page);
    if (index === -1) return null;
    return this.pages.value.at(index + offset) ?? null;
  }

  public async load() {
    this.#ready.value &&= false;
    this.#book.value = await getCurrentReaderBook();
    this.#ready.value ||= true;
  }
}
