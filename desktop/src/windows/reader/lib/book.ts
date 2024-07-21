import { pull } from 'lodash-es';
import { BookPageStatus, ReaderBookPageImpl, isNotStarted } from './page';

export class ReaderBookImpl implements Omit<ReaderBook, 'pages'> {
  public readonly id?: number;
  public readonly title: string;
  public readonly path: string;

  readonly #pages = new Map<number, ReaderBookPageImpl>();

  // Normally, we fetch pages in order, from the first to the last.
  // However, if the user jumps to a page, we need to prioritize fetching that page.
  // Pages in this stack have said priority.
  readonly #stack: number[] = [0];

  constructor(book: ReaderBook) {
    this.id = book.id;
    this.title = book.title;
    this.path = book.path;

    for (const page of book.pages) {
      this.#pages.set(page.index, new ReaderBookPageImpl(page));
    }
  }

  public indices() {
    const indices = Array.from(this.#pages.keys());
    indices.sort((a, b) => a - b);
    return indices;
  }

  public has(index: number) {
    return this.#pages.has(index);
  }

  public get(index: number) {
    const page = this.#pages.get(index);
    if (page?.status === BookPageStatus.NotStarted) {
      this.#stack.push(index);
    }

    return page;
  }

  public removePage(name: string) {
    const page = this.#pages.values().find((it) => it.name === name);
    if (page) {
      pull(this.#stack, page.index);
      return this.#pages.delete(page.index);
    }

    return false;
  }

  public async *fetch() {
    while (this.#pages.values().some(isNotStarted)) {
      if (this.#stack.length > 0) {
        const index = this.#stack.pop();
        if (typeof index === 'number') {
          const page = this.#pages.get(index);
          if (page) {
            yield page.fetch();
            continue;
          }
        }
      }

      const page = this.#pages.values().find(isNotStarted);
      if (page) yield page.fetch();
    }
  }

  get size() {
    return this.#pages.size;
  }
}
