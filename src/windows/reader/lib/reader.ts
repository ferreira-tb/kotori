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

    for await (const index of this.#book.fetch()) {
      if (this.#book.has(index)) {
        this.#trigger();
      }
    }
  }

  private go(page: number) {
    if (this.#book) {
      const size = this.#book.size;
      this.#current = ((page % size) + size) % size;

      // The page might have been deleted.
      if (this.#book.has(this.#current)) {
        this.#trigger();
        return true;
      }
    }

    return false;
  }

  public next() {
    if (this.#book) {
      let ok = false;
      while (!ok && this.#book.size > 0) {
        ok = this.go(this.#current + 1);
      }
    }
  }

  public previous() {
    if (this.#book) {
      let ok = false;
      while (!ok && this.#book.size > 0) {
        ok = this.go(this.#current - 1);
      }
    }
  }

  public first() {
    if (this.#book) {
      let ok = this.go(0);
      if (!ok) {
        const indices = this.#book.indices();
        while (!ok && this.#book.size > 0 && indices.length > 0) {
          const index = indices.shift();
          if (typeof index === 'number') {
            ok = this.go(index);
          }
        }
      }
    }
  }

  public last() {
    if (this.#book) {
      const last = this.#book.size - 1;
      let ok = this.go(last >= 0 ? last : 0);
      if (!ok) {
        const indices = this.#book.indices();
        while (!ok && this.#book.size > 0 && indices.length > 0) {
          const index = indices.pop();
          if (typeof index === 'number') {
            ok = this.go(index);
          }
        }
      }
    }
  }

  public removePage(name: string) {
    if (this.#book?.removePage(name)) {
      this.next();
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
        set: noop,
        get() {
          track();
          return reader;
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
