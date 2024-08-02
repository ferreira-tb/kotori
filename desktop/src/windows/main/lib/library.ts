import * as commands from '@/lib/commands';

class LibraryBookImpl implements LibraryBook {
  public readonly id: number;
  public readonly title: string;
  public readonly path: string;

  #cover?: Nullish<string>;
  #rating: number;
  #read: boolean;

  /** Used to force reload the image. */
  #version = 0;

  constructor(book: LibraryBook) {
    this.id = book.id;
    this.title = book.title;
    this.path = book.path;
    this.#rating = book.rating;
    this.#read = book.read;

    if (book.cover) {
      this.#cover = convertFileSrc(book.cover);
    }
  }

  get cover() {
    return this.#cover;
  }

  set cover(path: Nullish<string>) {
    if (!path) return;
    try {
      // Adds a version search parameter to force reload the image.
      const url = new URL(convertFileSrc(path));
      url.searchParams.set('v', (++this.#version).toString(10));
      this.#cover = url.toString();
    }
    catch {
      this.#cover = convertFileSrc(path);
    }
  }

  get rating() {
    return this.#rating;
  }

  set rating(rating: number) {
    if (Number.isInteger(rating) && rating >= 0 && rating <= 5) {
      this.#rating = rating;
    }
  }

  get read() {
    return this.#read;
  }

  set read(read: boolean) {
    this.#read = read;
  }

  public open() {
    commands.openBook(this.id).catch(handleError);
  }
}

export class Library {
  readonly #books = new Map<number, LibraryBookImpl>();
  readonly #trigger: Fn;

  private constructor(trigger: Fn) {
    this.#trigger = trigger;
  }

  public add(book: LibraryBook) {
    this.#books.set(book.id, new LibraryBookImpl(book));
    this.#trigger();
  }

  public find(id: number) {
    return this.#books.get(id);
  }

  public async load() {
    const books = await commands.getLibraryBooks();
    this.#books.clear();

    for (const book of books) {
      this.#books.set(book.id, new LibraryBookImpl(book));
    }

    this.#trigger();
  }

  public remove(id: number) {
    if (this.#books.delete(id)) {
      this.#trigger();
    }
  }

  public setBookCover(id: number, path: string) {
    const book = this.#books.get(id);
    if (book) {
      book.cover = path;
      this.#trigger();
    }
  }

  public setBookRating(id: number, rating: number) {
    const book = this.#books.get(id);
    if (book) {
      book.rating = rating;
      this.#trigger();
    }
  }

  public setBookRead(id: number, read: boolean) {
    const book = this.#books.get(id);
    if (book) {
      book.read = read;
      this.#trigger();
    }
  }

  public readonly iterator = {
    all: this.all.bind(this),
    favorites: this.favorites.bind(this),
    read: this.read.bind(this),
  };

  private *all() {
    for (const book of this.#books.values()) {
      yield book;
    }
  }

  private *favorites() {
    for (const book of this.#books.values()) {
      if (book.rating >= 4) yield book;
    }
  }

  private *read() {
    for (const book of this.#books.values()) {
      if (book.read) yield book;
    }
  }

  get size() {
    return this.#books.size;
  }

  public static createRef() {
    return customRef((track, trigger) => {
      const library = new Library(trigger);
      return {
        get() {
          track();
          return library;
        },
        set(_) {
          noop();
        },
      };
    });
  }
}

export type { LibraryBookImpl };
