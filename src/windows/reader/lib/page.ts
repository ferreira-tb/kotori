/* eslint-disable perfectionist/sort-enums */
import { Reader } from './reader';
import { getBookPage } from '@/lib/server';
import { deletePageWithDialog } from '@/lib/commands';

export enum BookPageStatus {
  NotStarted = 1,
  Pending = 2,
  Done = 3,
  Error = 4
}

export class BookPage {
  #status = BookPageStatus.NotStarted;
  #url: string | null = null;

  constructor(public readonly id: number) {}

  public async fetch() {
    if (this.#status === BookPageStatus.NotStarted) {
      try {
        this.#status = BookPageStatus.Pending;
        const blob = await getBookPage(Reader.windowId, this.id);
        this.#url = URL.createObjectURL(blob);
        this.#status = BookPageStatus.Done;
      } catch (err) {
        this.#status = BookPageStatus.Error;
        handleError(err);
      }
    }
  }

  public async delete() {
    try {
      await deletePageWithDialog(Reader.windowId, this.id);
    } catch (err) {
      handleError(err);
    }
  }

  get status() {
    return this.#status;
  }

  get url() {
    return this.#url;
  }
}

export function isNotStarted(page: BookPage) {
  return page.status === BookPageStatus.NotStarted;
}
