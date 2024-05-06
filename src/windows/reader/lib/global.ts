declare global {
  interface Window {
    readonly KOTORI: {
      readonly readerWindowId: number;
    };
  }
}

export const READER_WINDOW_ID = window.KOTORI.readerWindowId;
