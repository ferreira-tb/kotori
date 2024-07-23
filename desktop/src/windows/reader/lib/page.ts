/* eslint-disable perfectionist/sort-enums */
import { Reader } from './reader';
import { getBookPage } from '@/lib/server';
import { deletePageWithDialog } from '@/lib/commands';

export const enum BookPageStatus {
  NotStarted = 1,
  Pending = 2,
  Done = 3,
  Error = 4,
}

export class ReaderBookPageImpl implements ReaderBookPage {
  readonly #page: ReaderBookPage;
  #status = BookPageStatus.NotStarted;
  #url: string | null = null;

  constructor(page: ReaderBookPage) {
    this.#page = page;
  }

  public async fetch() {
    if (this.#status === BookPageStatus.NotStarted) {
      try {
        this.#status = BookPageStatus.Pending;
        const blob = await getBookPage(Reader.windowId, this.name);
        this.#url = URL.createObjectURL(blob);
        this.#status = BookPageStatus.Done;
      }
      catch (err) {
        this.#status = BookPageStatus.Error;
        handleError(err);
      }
    }

    return this.#page.index;
  }

  public delete() {
    return deletePageWithDialog(Reader.windowId, this.name);
  }

  get index() {
    return this.#page.index;
  }

  get name() {
    return this.#page.name;
  }

  get status() {
    return this.#status;
  }

  get url() {
    return this.#url;
  }
}

export function isNotStarted(page: ReaderBookPageImpl) {
  return page.status === BookPageStatus.NotStarted;
}
