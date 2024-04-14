export enum Command {
  AddToLibraryFromDialog = 'add_to_library_from_dialog',
  CloseCurrentWindow = 'close_current_window',
  DeleteBookPage = 'delete_book_page',
  FocusMainWindow = 'focus_main_window',
  GetCurrentReaderBook = 'get_current_reader_book',
  GetCurrentReaderWindowId = 'get_current_reader_window_id',
  GetLibraryBooks = 'get_library_books',
  OpenBook = 'open_book',
  OpenBookFromDialog = 'open_book_from_dialog',
  RemoveBook = 'remove_book',
  RequestDeletePage = 'request_delete_page',
  RequestRemoveBook = 'request_remove_book',
  ShowLibraryBookContextMenu = 'show_library_book_context_menu',
  ShowReaderPageContextMenu = 'show_reader_page_context_menu',
  SwitchReaderFocus = 'switch_reader_focus',
  UpdateBookRating = 'update_book_rating'
}

export function requestDeletePage(windowId: Nullish<number>, page: Nullish<number>) {
  if (typeof windowId === 'number' && typeof page === 'number') {
    invoke(Command.RequestDeletePage, { windowId, page }).catch(handleError);
  }
}

export function requestRemoveBook(bookId: Nullish<number>) {
  if (typeof bookId === 'number') {
    invoke(Command.RequestRemoveBook, { id: bookId }).catch(handleError);
  }
}

export function showLibraryBookContextMenu(bookId: number) {
  invoke(Command.ShowLibraryBookContextMenu, { id: bookId }).catch(handleError);
}

export function showReaderPageContextMenu(windowId: Nullish<number>, page: Nullish<number>) {
  if (typeof windowId === 'number' && typeof page === 'number') {
    invoke(Command.ShowReaderPageContextMenu, { windowId, page }).catch(handleError);
  }
}

export function openBook(bookId: number) {
  invoke(Command.OpenBook, { id: bookId }).catch(handleError);
}

export function updateBookRating(bookId: number, rating: number) {
  invoke(Command.UpdateBookRating, { id: bookId, rating }).catch(handleError);
}
