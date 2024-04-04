import { api } from '@/utils/server';
import { handleError } from 'manatsu';

export class Page {
  public status: BookStatus = 'not started';
  public url: string | null = null;

  private constructor(public readonly id: number) {}

  public async fetch() {
    try {
      if (this.status !== 'not started') return;
      this.status = 'pending';

      const readerId = window.__KOTORI__.readerId;
      const response = await fetch(api(`/reader/${readerId}/${this.id}`));
      const blob = await response.blob();

      this.url = URL.createObjectURL(blob);
      this.status = 'done';
    } catch (err) {
      this.status = 'error';
      handleError(err);
    }
  }

  public eagerFetch(pages: Page[]) {
    if (pages.length === 0) return;

    const promises: Promise<void>[] = [];
    if (this.status === 'not started') {
      promises.push(this.fetch());
    }

    for (let i = 1; i <= 5; i++) {
      const nextIndex = findIndex(pages, this.id) + i;
      if (pages[nextIndex] && pages[nextIndex].status === 'not started') {
        promises.push(pages[nextIndex].fetch());
      }
    }

    Promise.all(promises).catch(handleError);
  }

  public revoke() {
    if (this.url) {
      URL.revokeObjectURL(this.url);
      this.url = null;
    }
  }

  public static from(page: number): Page;
  public static from(pages: number[]): Page[];
  public static from(source: number | number[]): Page | Page[] {
    if (Array.isArray(source)) {
      return source.map((id) => new Page(id));
    }

    return new Page(source);
  }
}

function findIndex(pages: Page[], id: number) {
  return pages.findIndex((page) => page.id === id);
}
