import * as commands from './commands';

export function setGlobalSensors() {
  onKeyDown('Tab', commands.switchReaderFocus);
  onKeyDown('F11', commands.toggleFullscreen);

  onCtrlKeyDown('o', commands.openBookWithDialog);
  onCtrlShiftKeyDown(['a', 'A'], commands.addToLibraryWithDialog);
}
