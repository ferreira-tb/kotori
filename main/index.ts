import * as path from 'node:path';
import { app, BrowserWindow, Menu, nativeImage, shell, Tray } from 'electron';
import { setAppEvents } from './events';

declare const MAIN_WINDOW_VITE_DEV_SERVER_URL: string;
declare const MAIN_WINDOW_VITE_NAME: string;

function createWindow(icon: string) {
    const mainWindow = new BrowserWindow({
        width: 1400,
        height: 800,
        show: false,
        title: 'Kotori',
        icon,
        frame: true,
        resizable: true,
        movable: true,
        minimizable: true,
        maximizable: true,
        fullscreenable: true,
        closable: true,
        titleBarStyle: 'default',
        autoHideMenuBar: true,
        darkTheme: true,
        webPreferences: {
            spellcheck: false,
            nodeIntegration: true,
            nodeIntegrationInWorker: true,
            contextIsolation: false,
            webSecurity: true
        }
    });

    mainWindow.webContents.on('will-navigate', (e) => e.preventDefault());
    mainWindow.webContents.setWindowOpenHandler(() => ({ action: 'deny' }));

    const log = path.join(app.getPath('userData'), 'error.log');
    const template: Electron.MenuItemConstructorOptions[] = [
        { label: 'Reload', accelerator: 'F5', role: 'forceReload' },
        {
            label: 'Log',
            accelerator: 'F11',
            click: () => void shell.openPath(log)
        },
        { label: 'Inspect', accelerator: 'F12', role: 'toggleDevTools' },
        { label: 'Exit', accelerator: 'Esc', role: 'quit' }
    ];

    const menu = Menu.buildFromTemplate(template);
    Menu.setApplicationMenu(menu);

    if (MAIN_WINDOW_VITE_DEV_SERVER_URL) {
        mainWindow.loadURL(MAIN_WINDOW_VITE_DEV_SERVER_URL);
    } else {
        const file = path.join(
            __dirname,
            `../renderer/${MAIN_WINDOW_VITE_NAME}/index.html`
        );
        mainWindow.loadFile(file);
    }

    setAppEvents();

    mainWindow.once('ready-to-show', () => {
        mainWindow.show();
    });

    return mainWindow;
}

app.on('window-all-closed', () => {
    if (process.platform !== 'darwin') app.quit();
});

app.whenReady().then(() => {
    const icon = path.join(__dirname, 'favicon.ico');
    const mainWindow = createWindow(icon);

    app.on('activate', () => {
        if (BrowserWindow.getAllWindows().length === 0) {
            createWindow(icon);
        }
    });

    const contextMenu = Menu.buildFromTemplate([
        { label: 'Quit', role: 'quit' }
    ]);

    const tray = new Tray(nativeImage.createFromPath(icon));
    tray.setToolTip('Kotori');
    tray.setContextMenu(contextMenu);

    tray.on('click', () => {
        if (mainWindow.isMinimized()) mainWindow.restore();
    });
});
