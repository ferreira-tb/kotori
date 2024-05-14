import { useReaderStore } from '../stores';
import { getBookPage } from '@/lib/server';
import { READER_WINDOW_ID } from './global';
import { deletePageWithDialog } from '@/lib/commands';

export class Page {
  public status: ReaderBookStatus = 'not started';
  public url: string | null = null;

  private static lookahead = 5;
  private static lookbehind: number | null = -1;

  constructor(public readonly id: number) {}

  public async fetch() {
    try {
      if (this.status !== 'not started') return;
      console.log('Fetching page', this.id);
      this.status = 'pending';
      const blob = await getBookPage(READER_WINDOW_ID, this.id);
      this.url = URL.createObjectURL(blob);
      this.status = 'done';
    } catch (err) {
      this.status = 'error';
      handleError(err);
    }
  }

  public eagerFetch() {
    const { pages, findNextIndex, lastIndex } = useReaderStore();
    if (pages.length === 0) return;

    const promises: Promise<void>[] = [];
    if (this.status === 'not started') {
      promises.push(this.fetch());
    }

    if (Page.lookahead > 0) {
      for (let offset = 1; offset <= Page.lookahead; offset++) {
        const next = findNextIndex(this.id, offset);
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

    // Errors are already handled inside `fetch`.
    void Promise.all(promises);
  }

  public async delete() {
    try {
      await deletePageWithDialog(READER_WINDOW_ID, this.id);
    } catch (err) {
      handleError(err, { dialog: true });
    }
  }
}
