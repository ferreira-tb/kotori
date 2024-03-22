import { router } from '@/router';

export async function openFile() {
  const book = await invoke<Book | null>(Command.OpenFile);
  if (book) {
    const readerStore = useReaderStore();
    readerStore.book = convertBookSrc(book);
    await nextTick();
    await router.push('/reader');
  }
}

function convertBookSrc(book: Book): Book {
  book.path = convertFileSrc(book.path);
  book.tempDir = convertFileSrc(book.tempDir);
  book.pages = book.pages.map((page) => {
    page.path = convertFileSrc(page.path);
    return page;
  });

  return book;
}
