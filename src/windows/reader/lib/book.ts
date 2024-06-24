import { BookPage, BookPageStatus, isNotStarted } from './page';

export class ReaderBookImpl implements Omit<ReaderBook, 'pages'> {
  public readonly id?: number;
  public readonly title: string;
  public readonly path: string;

  readonly #pages = new Map<number, BookPage>();
  readonly #stack: number[] = [0];

  constructor(book: ReaderBook) {
    this.id = book.id;
    this.title = book.title;
    this.path = book.path;

    for (const page of book.pages) {
      this.#pages.set(page, new BookPage(page));
    }
  }

  public get(index: number) {
    const page = this.#pages.get(index);
    if (page?.status === BookPageStatus.NotStarted) {
      this.#stack.push(index);
    }

    return page;
  }

  public async *fetch() {
    while (this.#pages.values().some(isNotStarted)) {
      if (this.#stack.length > 0) {
        const index = this.#stack.pop();
        if (typeof index === 'number') {
          const page = this.#pages.get(index);
          if (page) yield page.fetch();
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
