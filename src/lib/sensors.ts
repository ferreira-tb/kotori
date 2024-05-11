import {
  addToLibraryFromDialog,
  openBookFromDialog,
  switchReaderFocus,
  toggleFullscreen
} from './commands';

function disableDefaultSensors() {
  preventContextMenu();
  preventKeyDown(['F3', 'F7']);

  // Focus move.
  preventKeyDown('Tab', { shiftKey: true });
}

export function setGlobalSensors() {
  disableDefaultSensors();

  onKeyDown('Tab', switchReaderFocus);
  onKeyDown('F11', toggleFullscreen);

  onCtrlKeyDown('o', openBookFromDialog);
  onCtrlShiftKeyDown(['a', 'A'], addToLibraryFromDialog);
}
