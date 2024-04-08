import { handleError } from 'manatsu';
import { invoke } from '@tauri-apps/api/core';

export enum Command {
  AddToLibraryFromDialog = 'add_to_library_from_dialog',
  CloseCurrentWindow = 'close_current_window',
  FocusMainWindow = 'focus_main_window',
  GetCurrentReaderBook = 'get_current_reader_book',
  GetCurrentReaderWindowId = 'get_current_reader_window_id',
  GetLibraryBooks = 'get_library_books',
  OpenBook = 'open_book',
  OpenBookFromDialog = 'open_book_from_dialog',
  SwitchReaderFocus = 'switch_reader_focus',
  UpdateBookRating = 'update_book_rating'
}

export function openBook(id: number) {
  invoke(Command.OpenBook, { id }).catch(handleError);
}

export function updateBookRating(id: number, rating: number) {
  invoke(Command.UpdateBookRating, { id, rating }).catch(handleError);
}
