declare global {
  interface Window {
    readonly KOTORI?: {
      readonly readerWindowId?: number;
    };
  }

  interface Iterator<T> {
    every: (fn: (value: T, index: number) => unknown) => boolean;
    find: (fn: (value: T, index: number) => unknown) => T | undefined;
    some: (fn: (value: T, index: number) => unknown) => boolean;
    toArray: () => T[];
  }
}

export {};
