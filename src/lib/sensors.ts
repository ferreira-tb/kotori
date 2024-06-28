import {
  addToLibraryWithDialog,
  openBookFromDialog,
  switchReaderFocus,
  toggleFullscreen
} from './commands';

function disableDefaultSensors() {
  preventContextMenu();
  preventKeyDown(['F3', 'F7']);

  // Search
  preventCtrlKeyDown(['f', 'F']);

  // Focus move.
  preventShiftKeyDown('Tab');
}

export function setGlobalSensors() {
  disableDefaultSensors();

  onKeyDown('Tab', switchReaderFocus);
  onKeyDown('F11', toggleFullscreen);

  onCtrlKeyDown('o', openBookFromDialog);
  onCtrlShiftKeyDown(['a', 'A'], addToLibraryWithDialog);
}
