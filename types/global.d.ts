import type { Nullish } from '@tb-dev/utility-types';

interface Kotori {
  readonly readerId: Nullish<number>;
}

declare global {
  interface Window {
    readonly __KOTORI__: Kotori;
  }
}
