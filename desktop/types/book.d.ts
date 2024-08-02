interface LibraryBook {
  cover?: Nullish<string>;
  readonly id: number;
  readonly path: string;
  rating: number;
  read: boolean;
  readonly title: string;
}

interface ReaderBook {
  readonly id?: number;
  readonly pages: ReaderBookPage[];
  readonly path: string;
  readonly title: string;
}

interface ReaderBookPage {
  readonly index: number;
  readonly name: string;
}

type ReaderBookStatus = 'not started' | 'pending' | 'done' | 'error';
