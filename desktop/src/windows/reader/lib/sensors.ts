import { setGlobalSensors } from '@/lib/sensors';
import { closeWindow, focusMainWindow } from '@/lib/commands';

export function setSensors() {
  setGlobalSensors();

  onKeyDown('Escape', closeWindow);
  onKeyDown('F1', focusMainWindow);
}
