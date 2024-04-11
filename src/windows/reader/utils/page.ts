import { getBookPage } from '@/utils/server';
import { useReaderStore } from '../stores';

export class Page {
  public status: ReaderBookStatus = 'not started';
  public url: string | null = null;

  private static lookahead = 5;
  private static lookbehind: number | null = -1;

  constructor(public readonly id: number) {}

  public async fetch() {
    try {
      if (this.status !== 'not started') return;
      this.status = 'pending';

      const { windowId } = useReaderStore();
      if (typeof windowId !== 'number') {
        throw new TypeError('window id is not available');
      }

      const blob = await getBookPage(windowId, this.id);
      this.url = URL.createObjectURL(blob);
      this.status = 'done';
    } catch (err) {
      this.status = 'error';
      handleError(err);
    }
  }

  public eagerFetch() {
    const { pages, findNext, lastIndex } = useReaderStore();
    if (pages.length === 0) return;

    const promises: Promise<void>[] = [];
    if (this.status === 'not started') {
      promises.push(this.fetch());
    }

    if (Page.lookahead > 0) {
      for (let step = 1; step <= Page.lookahead; step++) {
        const next = findNext(this.id, step);
        if (next && next.status === 'not started') {
          promises.push(next.fetch());
        }
      }

      const pagesUntilLast = lastIndex() - this.id;
      Page.lookahead = Math.min(pagesUntilLast, Page.lookahead + 1);
    }

    if (Page.lookbehind && Math.abs(Page.lookbehind) < pages.length) {
      const behind = pages.at(Page.lookbehind);
      if (behind) {
        if (behind.status === 'not started') {
          promises.push(behind.fetch());
          Page.lookbehind--;
        } else {
          Page.lookbehind = null;
        }
      }
    }

    Promise.all(promises).catch(handleError);
  }

  public static revokeAll() {
    const { pages } = useReaderStore();
    for (const page of pages) {
      if (page.url) {
        URL.revokeObjectURL(page.url);
        page.url = null;
      }
    }
  }
}
