export enum Command {
  AddToLibraryWithDialog = 'add_to_library_with_dialog',
  CloseWindow = 'close_window',
  DeletePageWithDialog = 'delete_page_with_dialog',
  FocusMainWindow = 'focus_main_window',
  GetCollections = 'get_collections',
  GetCurrentReaderBook = 'get_current_reader_book',
  GetLibraryBooks = 'get_library_books',
  OpenBook = 'open_book',
  OpenBookWithDialog = 'open_book_with_dialog',
  RemoveBook = 'remove_book',
  RemoveBookWithDialog = 'remove_book_with_dialog',
  ServerPort = 'server_port',
  ShowLibraryBookContextMenu = 'show_library_book_context_menu',
  ShowReaderPageContextMenu = 'show_reader_page_context_menu',
  ShowWindow = 'show_window',
  SwitchReaderFocus = 'switch_reader_focus',
  ToggleFullscreen = 'toggle_fullscreen',
  ToggleReaderMenu = 'toggle_reader_menu',
  UpdateBookRating = 'update_book_rating'
}

export async function addToLibraryWithDialog() {
  await invoke(Command.AddToLibraryWithDialog);
}

export function closeWindow() {
  invoke(Command.CloseWindow).catch(handleError);
}

export async function deletePageWithDialog(windowId: number, name: string) {
  await invoke(Command.DeletePageWithDialog, { windowId, name });
}

export function focusMainWindow() {
  invoke(Command.FocusMainWindow).catch(handleError);
}

export function getCurrentReaderBook(windowId: number) {
  return invoke<ReaderBook>(Command.GetCurrentReaderBook, { windowId });
}

export function getLibraryBooks() {
  return invoke<LibraryBook[]>(Command.GetLibraryBooks);
}

export function getServerPort() {
  return invoke<number>(Command.ServerPort);
}

export async function removeBook(id: number) {
  await invoke(Command.RemoveBook, { id });
}

export async function removeBookWithDialog(id: Nullish<number>) {
  if (typeof id !== 'number') return;
  await invoke(Command.RemoveBookWithDialog, { id });
}

export function showLibraryBookContextMenu(bookId: number) {
  invoke(Command.ShowLibraryBookContextMenu, { bookId }).catch(handleError);
}

export function showReaderPageContextMenu(windowId: number, name: string) {
  invoke(Command.ShowReaderPageContextMenu, { windowId, name }).catch(handleError);
}

export async function showWindow() {
  await invoke(Command.ShowWindow);
}

export async function switchReaderFocus() {
  await invoke(Command.SwitchReaderFocus);
}

export async function openBook(bookId: number) {
  await invoke(Command.OpenBook, { bookId });
}

export async function openBookWithDialog() {
  await invoke(Command.OpenBookWithDialog);
}

export function toggleFullscreen() {
  invoke(Command.ToggleFullscreen).catch(handleError);
}

export function toggleReaderMenu() {
  invoke(Command.ToggleReaderMenu).catch(handleError);
}

export function updateBookRating(bookId: number, rating: number) {
  invoke(Command.UpdateBookRating, { id: bookId, rating }).catch(handleError);
}
