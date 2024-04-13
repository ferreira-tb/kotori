import { Command } from '@/utils/commands';

export function disableDefaultSensors() {
  preventContextMenu();
  preventKeyDown(['F3', 'F7']);

  // Disable focus move.
  preventKeyDown('Tab', { shiftKey: true });
}

export function setGlobalSensors() {
  invokeOnKeyDown('Tab', Command.SwitchReaderFocus);
  invokeOnKeyDown('o', Command.OpenBookFromDialog, null, { ctrlKey: true });
  invokeOnKeyDown('A', Command.AddToLibraryFromDialog, null, { ctrlKey: true, shiftKey: true });
}
