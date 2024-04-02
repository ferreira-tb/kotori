interface Book {
  pages?: number[];
  path: string;
  title: string;
}

type BookStatus = 'not started' | 'pending' | 'done' | 'error';
