interface Book {
  pages: BookPage[];
  path: string;
  tempDir: string;
  title: string;
}

interface BookPage {
  filename: string;
  path: string;
}
