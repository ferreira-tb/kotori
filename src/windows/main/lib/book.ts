import { useLibraryStore } from '../stores';
import { removeBookWithDialog } from '@/lib/commands';

export function usePreview(): Ref<Nullish<LibraryBook>> {
  const library = useLibraryStore();
  const { books } = storeToRefs(library);

  const preview = ref<Nullish<LibraryBook>>(null);
  const stop = watchEffect(() => {
    if (books.value.every((book) => !book.cover)) {
      preview.value = null;
    } else if (preview.value && books.value.every(({ id }) => id !== preview.value?.id)) {
      preview.value = books.value.find((book) => book.cover);
    } else {
      preview.value ??= books.value.find((book) => book.cover);
    }
  });

  tryOnScopeDispose(() => stop());

  return preview;
}

export async function removeBook(id: Nullish<number>) {
  try {
    await removeBookWithDialog(id);
  } catch (err) {
    handleError(err, { dialog: true });
  }
}
