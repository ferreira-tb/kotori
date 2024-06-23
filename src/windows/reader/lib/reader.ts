import { ReaderBookImpl } from './book';
import { getCurrentReaderBook } from '@/lib/commands';

export class Reader {
  #book: Nullish<ReaderBookImpl>;
  readonly #trigger: Fn;

  #current = 0;

  private constructor(trigger: Fn) {
    this.#trigger = trigger;
  }

  public async load() {
    const book = await getCurrentReaderBook(Reader.windowId);
    this.#book = new ReaderBookImpl(book);
    this.#trigger();

    for await (const _ of this.#book.fetch()) {
      this.#trigger();
    }
  }

  public go(page: number) {
    if (this.#book) {
      const size = this.#book.size;
      this.#current = ((page % size) + size) % size;
      this.#trigger();
    }
  }

  public next() {
    this.go(this.#current + 1);
  }

  public previous() {
    this.go(this.#current - 1);
  }

  public first() {
    this.go(0);
  }

  public last() {
    if (this.#book) {
      const last = this.#book.size - 1;
      this.go(last >= 0 ? last : 0);
    }
  }

  get current() {
    return this.#book?.get(this.#current);
  }

  get size() {
    return this.#book?.size ?? 0;
  }

  public static createRef() {
    return customRef((track, trigger) => {
      const reader = new Reader(trigger);
      return {
        get() {
          track();
          return reader;
        },
        set(_newValue) {
          noop();
        }
      };
    });
  }

  static get windowId() {
    if (typeof window.KOTORI?.readerWindowId !== 'number') {
      throw new TypeError('missing reader window id');
    }

    return window.KOTORI.readerWindowId;
  }
}
