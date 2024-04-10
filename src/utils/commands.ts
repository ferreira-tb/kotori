import { handleError } from 'manatsu';
import { invoke } from '@tauri-apps/api/core';
import type { Nullish } from '@tb-dev/utility-types';

export enum Command {
  AddToLibraryFromDialog = 'add_to_library_from_dialog',
  CloseCurrentWindow = 'close_current_window',
  FocusMainWindow = 'focus_main_window',
  GetCurrentReaderBook = 'get_current_reader_book',
  GetCurrentReaderWindowId = 'get_current_reader_window_id',
  GetLibraryBooks = 'get_library_books',
  OpenBook = 'open_book',
  OpenBookFromDialog = 'open_book_from_dialog',
  ShowLibraryBookContextMenu = 'show_library_book_context_menu',
  ShowReaderPageContextMenu = 'show_reader_page_context_menu',
  SwitchReaderFocus = 'switch_reader_focus',
  UpdateBookRating = 'update_book_rating'
}

export function showLibraryBookContextMenu(id: number) {
  invoke(Command.ShowLibraryBookContextMenu, { id }).catch(handleError);
}

export function showReaderPageContextMenu(page: Nullish<number>) {
  if (typeof page !== 'number') return;
  invoke(Command.ShowReaderPageContextMenu, { page }).catch(handleError);
}

export function openBook(id: number) {
  invoke(Command.OpenBook, { id }).catch(handleError);
}

export function updateBookRating(id: number, rating: number) {
  invoke(Command.UpdateBookRating, { id, rating }).catch(handleError);
}
