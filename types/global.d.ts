import type { Nullish } from '@tb-dev/utility-types';

declare global {
  interface Window {
    readonly __KOTORI__: Kotori;
  }
}

interface Kotori {
  readonly readerId: Nullish<number>;
}
