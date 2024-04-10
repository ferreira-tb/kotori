interface LibraryBook {
  cover?: string;
  readonly id: number;
  readonly path: string;
  rating: number;
  readonly title: string;
}

interface ReaderBook {
  readonly id?: number;
  readonly pages?: number[];
  readonly path: string;
  readonly title: string;
}

type ReaderBookStatus = 'not started' | 'pending' | 'done' | 'error';
