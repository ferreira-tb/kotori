interface ReaderBook {
  pages?: number[];
  path: string;
  title: string;
}

type ReaderBookStatus = 'not started' | 'pending' | 'done' | 'error';
