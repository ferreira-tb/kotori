interface LibraryBook {
  cover?: string;
  id: number;
  path: string;
  rating: number;
  title: string;
}

interface ReaderBook {
  pages?: number[];
  path: string;
  title: string;
}

type ReaderBookStatus = 'not started' | 'pending' | 'done' | 'error';
