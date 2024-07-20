import {
  addToLibraryWithDialog,
  openBookWithDialog,
  switchReaderFocus,
  toggleFullscreen
} from './commands';

export function setGlobalSensors() {
  onKeyDown('Tab', switchReaderFocus);
  onKeyDown('F11', toggleFullscreen);

  onCtrlKeyDown('o', openBookWithDialog);
  onCtrlShiftKeyDown(['a', 'A'], addToLibraryWithDialog);
}
