export function convertBookSrc(book: Book): Book {
  book.path = convertFileSrc(book.path);
  book.tempDir = convertFileSrc(book.tempDir);
  book.pages = book.pages.map((page) => {
    page.path = convertFileSrc(page.path);
    return page;
  });

  return book;
}
