import { removeBookWithDialog } from '@/lib/commands';

export async function removeBook(id: Nullish<number>) {
  try {
    await removeBookWithDialog(id);
  } catch (err) {
    handleError(err);
  }
}
