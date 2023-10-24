import { app, ipcMain } from 'electron';

export function setAppEvents() {
    ipcMain.on('app(sync):user-data', (e) => {
        e.returnValue = app.getPath('userData');
    });

    ipcMain.handle('app:version', () => app.getVersion());
}