export enum Command {
  AddToLibraryFromDialog = 'add_to_library_from_dialog',
  CloseWindow = 'close_window',
  DeletePageWithDialog = 'delete_page_with_dialog',
  FocusMainWindow = 'focus_main_window',
  GetCollections = 'get_collections',
  GetCurrentReaderBook = 'get_current_reader_book',
  GetLibraryBooks = 'get_library_books',
  MaximizeWindow = 'maximize_window',
  OpenBook = 'open_book',
  OpenBookFromDialog = 'open_book_from_dialog',
  RemoveBook = 'remove_book',
  RemoveBookWithDialog = 'remove_book_with_dialog',
  ShowLibraryBookContextMenu = 'show_library_book_context_menu',
  ShowReaderPageContextMenu = 'show_reader_page_context_menu',
  ShowWindow = 'show_window',
  SwitchReaderFocus = 'switch_reader_focus',
  ToggleFullscreen = 'toggle_fullscreen',
  ToggleReaderMenu = 'toggle_reader_menu',
  UpdateBookRating = 'update_book_rating'
}

export async function addToLibraryFromDialog() {
  await invoke(Command.AddToLibraryFromDialog);
}

export function closeWindow() {
  invoke(Command.CloseWindow).catch(handleError);
}

export async function deletePageWithDialog(page: number) {
  await invoke(Command.DeletePageWithDialog, { page });
}

export function focusMainWindow() {
  invoke(Command.FocusMainWindow).catch(handleError);
}

export function getCurrentReaderBook() {
  return invoke<ReaderBook>(Command.GetCurrentReaderBook);
}

export function maximizeWindow() {
  invoke(Command.MaximizeWindow).catch(handleErrorWithDialog);
}

export async function removeBook(id: number) {
  await invoke(Command.RemoveBook, { id });
}

export async function removeBookWithDialog(id: Nullish<number>) {
  if (typeof id !== 'number') return;
  await invoke(Command.RemoveBookWithDialog, { id });
}

export function showLibraryBookContextMenu(bookId: number) {
  invoke(Command.ShowLibraryBookContextMenu, { id: bookId }).catch(handleError);
}

export function showReaderPageContextMenu(windowId: number, page: Nullish<number>) {
  if (typeof page === 'number') {
    invoke(Command.ShowReaderPageContextMenu, { windowId, page }).catch(handleError);
  }
}

export async function showWindow() {
  await invoke(Command.ShowWindow);
}

export async function switchReaderFocus() {
  await invoke(Command.SwitchReaderFocus);
}

export async function openBook(bookId: number) {
  await invoke(Command.OpenBook, { id: bookId });
}

export async function openBookFromDialog() {
  await invoke(Command.OpenBookFromDialog);
}

export function toggleFullscreen() {
  invoke(Command.ToggleFullscreen).catch(handleErrorWithDialog);
}

export function toggleReaderMenu() {
  invoke(Command.ToggleReaderMenu).catch(handleErrorWithDialog);
}

export function updateBookRating(bookId: number, rating: number) {
  invoke(Command.UpdateBookRating, { id: bookId, rating }).catch(handleError);
}
